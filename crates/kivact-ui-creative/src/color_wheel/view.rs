//! Role: The color_wheel component and its wheel, sliders, fields and swatches.

use super::data::ColorModel;
use super::geometry::{Channel, Geometry};
use super::state::{ColorOwner, Model, State};
use kivact::component;
use kivact::custom::part;
use kivact::geometry::Size;
use kivact::i18n::message;
use kivact::interaction::{DragCoordinates, DragPhase, DragUpdate, use_drag};
use kivact::state::{Callback, IntoValue, Signal, Value, memo};
use kivact::style::{Color, pct, style, use_theme};
use kivact::view::{
    A11yAction, A11yEvent, A11yRequest, EditPhase, Handled, Key, KeyEvent, Role, column, node,
    node_ref, show,
};

/// A hue wheel, saturation/value square, optional alpha strip, model fields
/// and optional swatches. Element modifiers apply to the whole group.
/// RGB fields use normalized channels, HSV uses hue degrees and normalized
/// saturation/value. HEX accepts #RRGGBB and #RRGGBBAA; invalid drafts stay
/// visible without writing, Enter confirms and Escape discards the draft.
/// Hidden alpha is preserved. Gray colors retain the last hue.
///
/// `model` is the channel model the fields show, which the model selector
/// writes without changing the color. `alpha` and `fields` show the alpha
/// strip and the model fields (both default true). `swatches` binds editable
/// swatches; local swatch selection is not an edit. `on_edit` observes each
/// continuous edit, including palette additions and removals. `wheel_size`
/// sets the canvas height, at least 96 pixels; non-finite values use 220.
///
/// Names use `color.wheel`, `color.model`, `color.hue`, `color.sv`,
/// `color.alpha_slider`, `color.red`, `color.green`, `color.blue`, `color.h`,
/// `color.s`, `color.v`, `color.alpha`, `color.hex`, `color.swatch`,
/// `color.add_swatch` and `color.remove_swatch`. Escape, pointer cancellation, hiding,
/// disabling and disposal restore the starting color, hue and palette before
/// Cancel. During a pointer drag, other channel edits are consumed.
#[component]
pub fn color_wheel(
    color: Signal<Color>,
    model: Signal<ColorModel>,
    #[prop(default = true)] alpha: Value<bool>,
    #[prop(default = true)] fields: Value<bool>,
    swatches: Option<Signal<Vec<Color>>>,
    on_edit: Option<Callback<EditPhase>>,
    #[prop(default = 220.0)] wheel_size: Value<f64>,
) -> impl Element {
    let root = column(())
        .role(Role::Group)
        .style(style().min_width(0).width(pct(100)).gap(8));
    let root_part = part(|input| input);
    let root_state = root_part.state();
    let blocked = memo(move || {
        let input = root_state.get();
        input.is_disabled() || input.is_hidden()
    });
    let drag = use_drag::<Channel>(DragCoordinates::Local);
    let state = Model::new(color, swatches, on_edit, blocked, drag);
    let value = color.into_value();
    let size = memo(move || {
        let size = wheel_size.get();
        if size.is_finite() {
            size.max(96.0)
        } else {
            220.0
        }
    });
    let reference = node_ref();
    let rect = reference.rect();
    let geometry = move || {
        Geometry::new(
            rect.get_untracked().map_or_else(
                || Size::new(size.get_untracked(), size.get_untracked()),
                |r| Size::new(r.width, r.height),
            ),
            alpha.get_untracked(),
        )
    };
    let active = memo(move || state.controls.get().channel);
    let gesture = wheel_gesture(state, drag, geometry, value);
    let state = State::new(state, gesture);
    let (canvas_part, group) = observed_canvas(state, alpha);
    let canvas = node()
        .role(Role::Canvas)
        .name(message("creative.color.wheel", "Color wheel"))
        .style(move || style().height(size.get()).min_width(0).width(pct(100)))
        .node_ref(reference)
        .disabled(state.blocked)
        .part(canvas_part)
        .part(gesture.source(Channel::Hue))
        .draw(move |draw, size| {
            let g = Geometry::new(size, alpha.get());
            g.regions(draw);
            super::geometry::paint(draw, g, value.get(), state.hue.get());
        })
        .on_key(move |event| key(state, active.get(), event));
    let canvas = canvas.regions(
        move || {
            let mut channels = vec![Channel::Hue, Channel::Sv];
            if alpha.get() {
                channels.push(Channel::Alpha);
            }
            channels
        },
        |channel| channel.id(),
        move |_, channel| channel_part(state, value, gesture, channel.get_untracked()),
    );
    root.node_ref(group).part(root_part).child((
        canvas,
        show(fields, move || super::fields::fields(state, model, alpha)),
        super::fields::swatches(state, alpha),
    ))
}

fn pair(text: &str) -> Option<(f64, f64)> {
    let mut parts = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty());
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    (parts.next().is_none() && x.is_finite() && y.is_finite()).then_some((x, y))
}

fn key(state: State, channel: Channel, event: KeyEvent) -> Handled {
    let theme = use_theme().get();
    let (x, y) = state.channel(channel, &theme);
    let delta = if channel == Channel::Hue { 1.0 } else { 0.01 }
        * if event.modifiers().shift { 10.0 } else { 1.0 };
    let next = match event.key() {
        Key::ArrowLeft => (x - delta, y),
        Key::ArrowRight => (x + delta, y),
        Key::ArrowUp if channel == Channel::Sv => (x, y + delta),
        Key::ArrowDown if channel == Channel::Sv => (x, y - delta),
        Key::ArrowUp => (x + delta, y),
        Key::ArrowDown => (x - delta, y),
        _ => return Handled::No,
    };
    state.once(ColorOwner::Wheel, || {
        state.set(channel, next.0, next.1, &theme)
    });
    Handled::Yes
}

/// Preserve the shared color session's wheel ownership throughout one pointer drag.
fn wheel_gesture(
    state: Model,
    drag: kivact::interaction::DragApi<Channel>,
    geometry: impl Fn() -> Geometry + Copy + 'static,
    value: Value<Color>,
) -> kivact::interaction::DragApi<Channel> {
    let apply = move |update: DragUpdate<Channel>| {
        if state.blocked.get_untracked() || state.edit.owner() != Some(ColorOwner::Wheel) {
            return;
        }
        let theme = use_theme().get();
        let (next, h) = geometry().update(
            update.payload,
            update.pos,
            value.get_untracked(),
            state.hue.get_untracked(),
            &theme,
        );
        state
            .controls
            .send(super::state::controls_machine::Event::Hue(h));
        state.color.set(next);
    };
    // Each drag is one edit of the shared session; Escape on the wheel,
    // an engine Cancel and `cancel` write the color from the press back.
    drag.on_start(move |update: DragUpdate<Channel>| {
        state.edit.begin(ColorOwner::Wheel, state.snapshot());
        state
            .controls
            .send(super::state::controls_machine::Event::Channel(
                update.payload,
            ));
        apply(update);
    })
    .on_update(apply)
    .on_end(move |update: DragUpdate<Channel>| {
        if update.phase == DragPhase::Cancel {
            state.edit.cancel_owned(ColorOwner::Wheel);
        } else {
            apply(update);
            state.edit.commit(ColorOwner::Wheel);
        }
    })
}
/// Attach one accessible color channel to the actual wheel and gesture authority.
fn channel_part(
    state: State,
    value: Value<Color>,
    gesture: kivact::interaction::DragApi<Channel>,
    channel: Channel,
) -> kivact::custom::Part<kivact::custom::PartState> {
    let (label, fallback) = match channel {
        Channel::Hue => ("creative.color.hue", "Hue"),
        Channel::Sv => ("creative.color.sv", "Saturation and value"),
        Channel::Alpha => ("creative.color.alpha_slider", "Alpha"),
    };
    let part = {
        let input_part = part(|input| input);
        let label = message(label, fallback);
        kivact::custom::Part::new(input_part.state(), move |node| {
            node.part(input_part)
                .role(Role::Slider)
                .focusable(true)
                .name(label)
                .value_text(move || {
                    value.get();
                    state.hue.get();
                    let (x, y) = state.channel(channel, &use_theme().get());
                    if channel == Channel::Sv {
                        format!("{x},{y}")
                    } else {
                        x.to_string()
                    }
                })
                .value_min(0.0)
                .value_max(if channel == Channel::Hue { 360.0 } else { 1.0 })
                .value_now(move || {
                    value.get();
                    state.hue.get();
                    state.channel(channel, &use_theme().get()).0
                })
        })
    };
    let target = kivact::view::node_ref();
    let part =
        kivact::custom::Part::new(part.state(), move |node| node.part(part).node_ref(target));
    let action = move |event| channel_action(state, channel, target, event);
    let drag_source = gesture.source(channel);
    kivact::custom::Part::new(part.state(), move |node| {
        node.part(part)
            .part(drag_source)
            .on_key(move |event| {
                state
                    .controls
                    .send(super::state::controls_machine::Event::Channel(channel));
                key(state, channel, event)
            })
            .a11y_action(A11yAction::Select, action)
            .a11y_action(A11yAction::Activate, action)
            .a11y_action(A11yAction::SetValue, action)
            .a11y_action(A11yAction::Increment, action)
            .a11y_action(A11yAction::Decrement, action)
    })
}
/// Apply accessible channel changes through the existing color edit transaction.
fn channel_action(state: State, channel: Channel, target: kivact::view::NodeRef, event: A11yEvent) {
    if matches!(event.request, A11yRequest::Select | A11yRequest::Activate) {
        state
            .controls
            .send(super::state::controls_machine::Event::Channel(channel));
        target.focus();
        return;
    }
    let theme = use_theme().get();
    let (x, y) = state.channel(channel, &theme);
    let step = if channel == Channel::Hue { 1.0 } else { 0.01 };
    let next = match &event.request {
        A11yRequest::SetValue(text) if channel == Channel::Sv => pair(text),
        A11yRequest::SetValue(text) => text
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .map(|x| (x, y)),
        A11yRequest::Increment => Some((x + step, y)),
        A11yRequest::Decrement => Some((x - step, y)),
        _ => None,
    };
    if let Some((x, y)) = next {
        state.once(ColorOwner::Wheel, || state.set(channel, x, y, &theme));
    }
}

/// Observe the actual canvas input and focus scope through the existing color state.
fn observed_canvas(
    state: State,
    alpha: Value<bool>,
) -> (
    kivact::custom::Part<kivact::custom::PartState>,
    kivact::view::NodeRef,
) {
    let canvas_part = {
        let input_part = part(|input| input);
        kivact::custom::Part::new(input_part.state(), move |node| {
            node.part(input_part).focusable(true)
        })
    };
    let canvas_state = canvas_part.state();
    let group = node_ref();
    state.observe(
        group.focus_within(),
        memo(move || {
            let input = canvas_state.get();
            input.is_disabled() || input.is_hidden()
        }),
        alpha,
    );
    (canvas_part, group)
}
