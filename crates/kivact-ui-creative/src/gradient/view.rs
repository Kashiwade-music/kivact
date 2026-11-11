//! Role: The GradientEditor builder and its ramp, stop handles and fields.

use super::data::GradientStop;
use super::data::position_value;
use super::geometry::Ramp;
use super::state::{GradientOwner, Model, State};
use kivact::custom::part;
use kivact::geometry::Size;
use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_drag};
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::state::callback;
use kivact::state::{Binding, Callback, binding, memo};
use kivact::style::{Color, pct, style};
use kivact::view::EditPhase;
use kivact::view::HandlerResult;
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, Handled, IntoView, Key, KeyEvent, Role, View, node_ref,
};
use kivact::view::{Element, Node, column, node, row};
use std::rc::Rc;

/// An inline ramp, selected position field and color wheel. Stops use vector indices.
/// Display sorting never reorders the application's vector. External invalid
/// positions are normalized for display without writing the binding.
///
/// Left/Down and Right/Up change position by 0.01 (Shift: 0.1), Home/End use
/// 0/1, Delete/Backspace remove a stop. Select/Activate change selection without
/// an edit. Other edits are suppressed during a stop drag. Escape, pointer
/// cancellation, hiding, disabling and disposal restore the exact original
/// vector and selection before Cancel. Names use `gradient.stops`,
/// `gradient.stop`, `gradient.position`, `gradient.add` and `gradient.remove`.
pub struct GradientEditor {
    root: Node,
    stops: Binding<Vec<GradientStop>>,
    selection: Binding<Option<usize>>,
    to_display: Rc<dyn Fn(Color) -> Color>,
    from_display: Rc<dyn Fn(Color) -> Color>,
    edit: Option<Callback<EditPhase>>,
}

/// Edit gradient `stops` by vector index; `selected` is the index of the
/// selected stop, which selecting alone does not report to `on_edit`. The
/// root name is set with Element::name.
#[track_caller]
pub fn gradient_editor(
    stops: impl IntoBinding<Vec<GradientStop>>,
    selected: impl IntoBinding<Option<usize>>,
) -> GradientEditor {
    GradientEditor {
        root: column(())
            .role(Role::Group)
            .style(style().gap(8).width(pct(100)).min_width(0)),
        stops: stops.into_binding(),
        selection: selected.into_binding(),
        to_display: Rc::new(|c| c),
        from_display: Rc::new(|c| c),
        edit: None,
    }
}

impl GradientEditor {
    /// Convert storage colors to display colors and writes back to storage.
    /// Capturing closures are supported. Opening and cancellation preserve raw
    /// storage values, including when these functions are not exact inverses.
    pub fn color_transform(
        mut self,
        to_display: impl Fn(Color) -> Color + 'static,
        from_display: impl Fn(Color) -> Color + 'static,
    ) -> Self {
        self.to_display = Rc::new(to_display);
        self.from_display = Rc::new(from_display);
        self
    }
    /// Observe one transaction for each drag, key, AT, button or child field edit.
    pub fn on_edit<R: HandlerResult>(mut self, edit: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.edit = Some(callback(edit));
        self
    }
}

impl Element for GradientEditor {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

fn key(state: State, id: usize, event: KeyEvent) -> Handled {
    let Some(stop) = state
        .stops
        .into_value()
        .with_untracked(|stops| stops.get(id).copied())
    else {
        return Handled::No;
    };
    let step = if event.modifiers().shift { 0.1 } else { 0.01 };
    let position = position_value(stop.position);
    match event.key() {
        Key::ArrowLeft | Key::ArrowDown => state.change_position(id, position - step),
        Key::ArrowRight | Key::ArrowUp => state.change_position(id, position + step),
        Key::Home => state.change_position(id, 0.0),
        Key::End => state.change_position(id, 1.0),
        Key::Delete | Key::Backspace => state.remove(id),
        _ => return Handled::No,
    };
    Handled::Yes
}

impl IntoView for GradientEditor {
    fn into_view(self) -> View {
        let stops = self.stops.into_value();
        let selection = self.selection;
        let selected = selection.into_value();
        let targets = crate::editors::shared::targets::targets::<usize, ()>(move || {
            Ok((0..stops.get().len()).collect())
        });
        let scene = targets.scene();
        let valid = memo(move || scene.get().is_ok());
        let root_part = part(|input| input);
        let input = root_part.state();
        let blocked = memo(move || {
            let input = input.get();
            input.is_disabled() || input.is_hidden()
        });
        let drag = use_drag::<(usize, f64, f64)>(DragCoordinates::Local);
        let state = Model::new(self.stops, selection, valid, self.edit, blocked, drag);
        let reference = node_ref();
        let rect = reference.rect();
        let apply = move |update: DragUpdate<(usize, f64, f64)>| {
            if state.enabled() && state.edit.owner() == Some(GradientOwner::Stop) {
                let (id, position, width) = update.payload;
                state.position(id, position + update.delta.x / width);
            }
        };
        // Each drag of a stop is one edit of the shared session; Escape, an
        // engine Cancel and `State::cancel` write the stops from the press
        // back. A press during a field's edit joins that edit and moves
        // nothing.
        let gesture = drag
            .on_start(move |update: DragUpdate<(usize, f64, f64)>| {
                state.edit.begin(GradientOwner::Stop, state.snapshot());
                if !state.enabled() || state.edit.owner() != Some(GradientOwner::Stop) {
                    return;
                }
                selection.set(Some(update.payload.0));
            })
            .on_update(apply)
            .on_end(move |update: DragUpdate<(usize, f64, f64)>| {
                if update.phase == DragPhase::Cancel {
                    state.edit.cancel_owned(GradientOwner::Stop);
                } else {
                    apply(update);
                    state.edit.commit(GradientOwner::Stop);
                }
            });
        let state = State::new(state, gesture);
        let group = node_ref();
        state.observe(group.focus_within());
        let disabled = memo(move || state.blocked.get() || !valid.get());
        let infos = stop_infos(stops, scene);
        let to_display = self.to_display.clone();
        let ramp = node()
            .role(Role::Canvas)
            .name(message("creative.gradient.stops", "Gradient stops"))
            .node_ref(reference)
            .style(style().height(44).width(pct(100)).min_width(0))
            .disabled(disabled)
            .draw(move |draw, size| {
                if let Ok(scene) = scene.get() {
                    let data = stops.get();
                    let ramp = Ramp::new(size);
                    ramp.paint(draw, &data, selected.get(), to_display.as_ref());
                    for (index, stop) in data.iter().enumerate() {
                        if let Some(id) = scene.id(&index) {
                            draw.region(id, ramp.hit(stop.position));
                        }
                    }
                }
            })
            .regions(
                infos,
                |(id, _)| *id,
                move |_, info| stop_part(state, gesture, rect, info.get_untracked().1),
            );
        self.root
            .node_ref(group)
            .part(root_part)
            .child((ramp, fields(state, self.to_display, self.from_display)))
            .into_view()
    }
}

fn fields(
    state: State,
    to_display: Rc<dyn Fn(Color) -> Color>,
    from_display: Rc<dyn Fn(Color) -> Color>,
) -> Node {
    let stops = state.stops.into_value();
    let selected = state.selection.into_value();
    let field_disabled = memo(move || {
        stops.get();
        selected.get();
        state.blocked.get() || !state.valid.get() || state.selected().is_none()
    });
    let position = binding(
        (move || {
            stops.get();
            selected.get();
            state.selected().map_or(0.0, |s| position_value(s.position))
        })
        .into_value(),
        move |position| {
            if state.enabled()
                && state.edit.owner() == Some(GradientOwner::Position)
                && let Some(index) = state.selection.into_value().get()
            {
                state.position(index, position);
            }
        },
    );
    let color = binding(
        (move || {
            stops.get();
            selected.get();
            to_display(state.selected().map_or(Color::WHITE, |s| s.color))
        })
        .into_value(),
        move |display| {
            if state.enabled() && state.edit.owner() == Some(GradientOwner::Color) {
                state.color(from_display(display));
            }
        },
    );
    let position = crate::number(
        message("creative.gradient.position", "Stop position"),
        position,
    )
    .range(
        kivact::interaction::ScalarRange::new(0.0, 1.0)
            .expect("invariant: finite ordered control range"),
    )
    .step(0.01)
    .precision(3)
    .with_node(move |node| node.disabled(field_disabled))
    .on_edit(move |phase| state.phase(GradientOwner::Position, phase));
    let buttons = row((
        kivact::ui::button(message("creative.gradient.add", "Add stop"))
            .with_node(move |node| node.disabled(move || state.blocked.get() || !state.valid.get()))
            .on_press(move || state.add()),
        kivact::ui::button(message("creative.gradient.remove", "Remove stop"))
            .with_node(move |node| node.disabled(field_disabled))
            .on_press(move || {
                if let Some(index) = state.selection.into_value().get() {
                    state.remove(index);
                }
            }),
    ))
    .style(style().gap(8));
    column((
        position,
        buttons,
        crate::color_wheel(color, kivact::state::signal(crate::ColorModel::Rgb))
            .alpha(true)
            .wheel_size(160.0)
            .with_node(move |node| node.disabled(field_disabled))
            .on_edit(move |phase| state.phase(GradientOwner::Color, phase)),
    ))
    .style(style().gap(8).min_width(0).width(pct(100)))
}

fn stop_part(
    state: State,
    gesture: kivact::interaction::DragApi<(usize, f64, f64)>,
    rect: kivact::state::Value<Option<kivact::geometry::Rect>>,
    id: usize,
) -> kivact::custom::Part<kivact::custom::PartState> {
    let stops = state.stops.into_value();

    let label = message("creative.gradient.stop", "Gradient stop");
    let part = {
        let input_part = part(|input| input);
        kivact::custom::Part::new(input_part.state(), move |node| {
            node.part(input_part)
                .role(Role::Slider)
                .focusable(true)
                .name(move || format!("{} {id}", label.get()))
                .value_min(0.0)
                .value_max(1.0)
                .value_now(move || {
                    stops.with(|stops| stops.get(id).map_or(0.0, |s| position_value(s.position)))
                })
                .value_text(move || {
                    stops
                        .with(|stops| stops.get(id).map_or(0.0, |s| position_value(s.position)))
                        .to_string()
                })
        })
    };
    let target = kivact::view::node_ref();
    let part =
        kivact::custom::Part::new(part.state(), move |node| node.part(part).node_ref(target));
    let action = move |event| stop_action(state, id, target, event);
    let drag_source = gesture.source_with(move || {
        let position = stops.with_untracked(|stops| {
            stops
                .get(id)
                .map_or(0.0, |stop| position_value(stop.position))
        });
        let size = rect
            .get_untracked()
            .map_or(Size::new(240.0, 44.0), |r| Size::new(r.width, r.height));
        (id, position, Ramp::new(size).width)
    });
    kivact::custom::Part::new(part.state(), move |node| {
        node.part(part)
            .part(drag_source)
            .on_key(move |event| key(state, id, event))
            .a11y_action(A11yAction::Select, action)
            .a11y_action(A11yAction::Activate, action)
            .a11y_action(A11yAction::SetValue, action)
            .a11y_action(A11yAction::Increment, action)
            .a11y_action(A11yAction::Decrement, action)
    })
}

fn stop_action(state: State, id: usize, target: kivact::view::NodeRef, event: A11yEvent) {
    let stops = state.stops.into_value();
    let selection = state.selection;

    if !state.enabled() {
        return;
    }
    if matches!(event.request, A11yRequest::Select | A11yRequest::Activate) {
        if !state.edit.is_active() {
            selection.set(Some(id));
            target.focus();
        }
        return;
    }
    let current =
        stops.with_untracked(|stops| stops.get(id).map_or(0.0, |s| position_value(s.position)));
    let next = match &event.request {
        A11yRequest::SetValue(text) => text.trim().parse::<f64>().ok().filter(|v| v.is_finite()),
        A11yRequest::Increment => Some(current + 0.01),
        A11yRequest::Decrement => Some(current - 0.01),
        _ => None,
    };
    if let Some(next) = next {
        state.change_position(id, next);
    }
}

type StopScene = kivact::state::Value<
    Result<std::rc::Rc<crate::editors::shared::targets::Scene<usize>>, kivact::UiError>,
>;
fn stop_infos(
    stops: kivact::state::Value<Vec<GradientStop>>,
    scene: StopScene,
) -> kivact::state::Value<Vec<(kivact::draw::RegionId, usize)>> {
    memo(move || {
        scene.with(|scene| {
            scene.as_ref().map_or_else(
                |_| Vec::new(),
                |scene| {
                    stops
                        .get()
                        .iter()
                        .enumerate()
                        .filter_map(|(index, _)| scene.id(&index).map(|id| (id, index)))
                        .collect()
                },
            )
        })
    })
    .into_value()
}
