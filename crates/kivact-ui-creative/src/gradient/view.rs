//! Role: The GradientEditor builder and its ramp, stop handles and fields.

use super::data::GradientStop;
use super::data::position_value;
use super::geometry::Ramp;
use super::state::{Domain, State};
use kivact::control::{Node, NodeBuilder, column, node, row};
use kivact::draw::Size;
use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_drag};
use kivact::primitive::part;
use kivact::style::{Color, pct, style};
use kivact::ui::EditPhase;
use kivact::view::HandlerResult;
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, Handled, IntoValue, IntoView, Key, KeyEvent, Role, View,
    callback, node_ref,
};
use kivact::{Binding, Callback, binding, effect, memo, var};
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

/// Edit gradient `stops` by vector index; `selection` is the index of the
/// selected stop, which selecting alone does not report to `on_edit`. The
/// root name is set with NodeBuilder::name.
#[track_caller]
pub fn gradient_editor(
    stops: impl Into<Binding<Vec<GradientStop>>>,
    selection: impl Into<Binding<Option<usize>>>,
) -> GradientEditor {
    GradientEditor {
        root: column(())
            .role(Role::Group)
            .style(style().gap(8).width(pct(100)).min_width(0)),
        stops: stops.into(),
        selection: selection.into(),
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

impl NodeBuilder for GradientEditor {
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
    let step = if event.modifiers.shift { 0.1 } else { 0.01 };
    let position = position_value(stop.position);
    match event.key {
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
        let scene =
            crate::editors::shared::targets::targets(move || Ok((0..stops.get().len()).collect()));
        let valid = memo(move || scene.get().is_ok());
        let state = State::new(self.stops, selection, valid, self.edit);
        let root_part = part(|input| input);
        let input = root_part.state();
        let reference = node_ref();
        let rect = reference.rect();
        let snapshot = var((0.0, 1.0));
        let apply = move |update: DragUpdate<usize>| {
            if state.enabled() && state.domain.get() == Some(Domain::Stop) {
                let (position, width) = snapshot.get();
                state.position(update.payload, position + f64::from(update.delta.x) / width);
            }
        };
        // Each drag of a stop is one edit of the shared session; Escape, an
        // engine Cancel and `State::cancel` write the stops from the press
        // back. A press during a field's edit joins that edit and moves
        // nothing.
        let gesture = use_drag::<usize>(DragCoordinates::Local)
            .edit(state.edit, move || state.snapshot())
            .on_start(move |update: DragUpdate<usize>| {
                if !state.enabled() || state.domain.get().is_some() {
                    return;
                }
                if let Some(stop) = stops.with(|stops| stops.get(update.payload).copied()) {
                    let size = rect
                        .get()
                        .map_or(Size::new(240.0, 44.0), |r| Size::new(r.width, r.height));
                    snapshot.set((position_value(stop.position), Ramp::new(size).width));
                    state.domain.set(Some(Domain::Stop));
                    selection.set(Some(update.payload));
                }
            })
            .on_update(apply)
            .on_end(move |update: DragUpdate<usize>| {
                if update.phase != DragPhase::Cancel {
                    apply(update);
                }
            });
        state.drag.set(Some(gesture));
        effect(move || {
            let input = input.get();
            let blocked = input.disabled || input.hidden;
            state.blocked.set(blocked);
            if blocked || !valid.get() {
                state.cancel();
            }
        });
        let disabled = memo(move || state.blocked.get() || !valid.get());
        let infos = memo(move || {
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
        });
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
                move |_, info| {
                    let id = info.get_untracked().1;
                    let label = message("creative.gradient.stop", "Gradient stop");
                    let part = part(|input| input)
                        .role(Role::Slider)
                        .focusable(true)
                        .name(move || format!("{} {id}", label.get()))
                        .value_min(0.0)
                        .value_max(1.0)
                        .value_now(move || {
                            stops.with(|stops| {
                                stops.get(id).map_or(0.0, |s| position_value(s.position))
                            })
                        })
                        .value_text(move || {
                            stops
                                .with(|stops| {
                                    stops.get(id).map_or(0.0, |s| position_value(s.position))
                                })
                                .to_string()
                        });
                    let target = part.node_ref();
                    let action = move |event: A11yEvent| {
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
                        let current = stops.with_untracked(|stops| {
                            stops.get(id).map_or(0.0, |s| position_value(s.position))
                        });
                        let next = match &event.request {
                            A11yRequest::SetValue(text) => {
                                text.trim().parse::<f64>().ok().filter(|v| v.is_finite())
                            }
                            A11yRequest::Increment => Some(current + 0.01),
                            A11yRequest::Decrement => Some(current - 0.01),
                            _ => None,
                        };
                        if let Some(next) = next {
                            state.change_position(id, next);
                        }
                    };
                    part.with(gesture.source(id))
                        .on_key(move |event| key(state, id, event))
                        .on_a11y_event(A11yAction::Select, action)
                        .on_a11y_event(A11yAction::Activate, action)
                        .on_a11y_event(A11yAction::SetValue, action)
                        .on_a11y_event(A11yAction::Increment, action)
                        .on_a11y_event(A11yAction::Decrement, action)
                },
            );
        self.root
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
                && state.domain.get() == Some(Domain::Position)
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
            if state.enabled()
                && state.domain.get() == Some(Domain::Color)
                && state.edit.is_active()
            {
                state.color(from_display(display));
            }
        },
    );
    let position = crate::number(
        message("creative.gradient.position", "Stop position"),
        position,
    )
    .range(0.0, 1.0)
    .step(0.01)
    .precision(3)
    .disabled(field_disabled)
    .on_edit(move |phase| state.phase(Domain::Position, phase));
    let buttons = row((
        kivact::ui::button(message("creative.gradient.add", "Add stop"))
            .disabled(move || state.blocked.get() || !state.valid.get())
            .on_press(move || state.add()),
        kivact::ui::button(message("creative.gradient.remove", "Remove stop"))
            .disabled(field_disabled)
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
        crate::color_wheel(color, kivact::signal(crate::ColorModel::Rgb))
            .alpha(true)
            .wheel_size(160.0)
            .disabled(field_disabled)
            .on_edit(move |phase| state.phase(Domain::Color, phase)),
    ))
    .style(style().gap(8).min_width(0).width(pct(100)))
}
