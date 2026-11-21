//! Role: The gradient_editor component and its ramp, stop handles and fields.

use super::data::ColorTransform;
use super::data::GradientStop;
use super::data::position_value;
use super::geometry::Ramp;
use super::state::{GradientOwner, Model, State};
use crate::color_wheel::{ColorModel, color_wheel};
use kivact::component;
use kivact::custom::part;
use kivact::geometry::Size;
use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_drag};
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::{Callback, binding, memo, signal};
use kivact::style::{Color, pct, style};
use kivact::view::EditPhase;
use kivact::view::{A11yAction, A11yEvent, A11yRequest, Handled, Key, KeyEvent, Role, node_ref};
use kivact::view::{Element, Node, column, node, row};
use std::rc::Rc;

/// An inline ramp, selected position field and color wheel. Stops use vector indices.
/// Display sorting never reorders the application's vector. External invalid
/// positions are normalized for display without writing the binding.
///
/// `selected` is the index of the selected stop; selecting alone is not an edit
/// and is not reported to `on_edit`. `color_transform` converts storage colors
/// for display and back. `on_edit` observes one transaction for each drag, key,
/// AT, button or child field edit.
///
/// Left/Down and Right/Up change position by 0.01 (Shift: 0.1), Home/End use
/// 0/1, Delete/Backspace remove a stop. Select/Activate change selection without
/// an edit. Other edits are suppressed during a stop drag. Escape, pointer
/// cancellation, hiding, disabling and disposal restore the exact original
/// vector and selection before Cancel. Names use `gradient.stops`,
/// `gradient.stop`, `gradient.position`, `gradient.add` and `gradient.remove`.
#[component]
pub fn gradient_editor(
    stops: Signal<Vec<GradientStop>>,
    selected: Signal<Option<usize>>,
    color_transform: Option<ColorTransform>,
    on_edit: Option<Callback<EditPhase>>,
) -> impl Element {
    let root = column(())
        .role(Role::Group)
        .style(style().gap(8).width(pct(100)).min_width(0));
    let identity: Rc<dyn Fn(Color) -> Color> = Rc::new(|color| color);
    let (to_display, from_display) = match color_transform {
        Some(transform) => (transform.to_display, transform.from_display),
        None => (identity.clone(), identity),
    };
    let selection = selected;
    let stop_values = stops.into_value();
    let selected = selection.into_value();
    let targets = crate::editors::shared::targets::targets::<usize, ()>(move || {
        Ok((0..stop_values.get().len()).collect())
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
    let state = Model::new(stops, selection, valid, on_edit, blocked, drag);
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
    let infos = stop_infos(stop_values, scene);
    let ramp_to_display = to_display.clone();
    let ramp = node()
        .role(Role::Canvas)
        .name(message("creative.gradient.stops", "Gradient stops"))
        .node_ref(reference)
        .style(style().height(44).width(pct(100)).min_width(0))
        .disabled(disabled)
        .draw(move |draw, size| {
            if let Ok(scene) = scene.get() {
                let data = stop_values.get();
                let ramp = Ramp::new(size);
                ramp.paint(draw, &data, selected.get(), ramp_to_display.as_ref());
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
    root.node_ref(group)
        .part(root_part)
        .child((ramp, fields(state, to_display, from_display)))
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
        color_wheel(color, signal(ColorModel::Rgb))
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
