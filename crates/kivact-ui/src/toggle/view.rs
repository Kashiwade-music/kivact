//! Role: The checkbox and switch components: boolean controls.

use super::switch_track;
use crate::control_size::{ControlSize, default_control_size};
use crate::helpers::{single_line_text, visible_label};
use crate::layer::with_base_recipe_style;
use crate::toggle::look::{CheckControl, CheckableState, CheckboxTheme, SwitchTheme};
use kivact_core::custom::{Caller, Part};
use kivact_core::state::{Callback, IntoValue, Signal, Value, memo};
use kivact_core::view::{CheckedState, EditPhase, node};
use kivact_interaction::{ToggleState, use_checkbox, use_switch};
use kivact_macros::component;

/// The live visual of a switch's opted-in drag: `None` when the drag is off.
type Visual = Option<Value<Option<kivact_interaction::ThumbDrag>>>;

/// A switch's opted-in drag on its control part, and the thumb it moves.
fn switch_gesture(
    control: Part<ToggleState>,
    value: Signal<bool>,
    size: Value<ControlSize>,
    on_edit: Option<Callback<EditPhase>>,
) -> (Part<ToggleState>, Visual) {
    let drag =
        kivact_interaction::use_switch_drag(value, move || switch_track::switch_track(size.get()));
    if !drag.is_enabled() {
        return (control, None);
    }
    if let Some(observer) = on_edit {
        drag.on_edit(move |phase| observer.call(phase));
    }
    (drag.control(control), Some(drag.thumb()))
}

/// The behavior of a check box, with its mixed state when it has one.
fn checkbox_behavior(
    state: Signal<bool>,
    mixed: Option<Value<bool>>,
) -> kivact_interaction::CheckboxApi {
    let behavior = use_checkbox(state);
    match mixed {
        Some(mixed) => behavior.indeterminate(mixed),
        None => behavior,
    }
}

/// A switch's localized word for its state, made at setup.
fn switch_text(on: Value<bool>) -> Value<String> {
    let (yes, no) = (
        kivact_core::i18n::message("switch.on", "ON"),
        kivact_core::i18n::message("switch.off", "OFF"),
    );
    memo(move || if on.get() { yes.get() } else { no.get() })
}

/// A switch's control: on or off, its localized word for the state (made at
/// setup) and the live visual of its opted-in drag.
fn switch_control(
    state: Signal<bool>,
    drag: Visual,
) -> impl Fn(CheckedState) -> CheckControl + 'static {
    let text = switch_text(state.into_value());
    move |checked| CheckControl::Switch {
        on: checked.is_checked(),
        text,
        drag: drag.and_then(|drag| drag.get()),
    }
}

/// A check box named and labelled `label` that edits `checked`.
///
/// It has `Role::CheckBox` and the checked state; Space and click flip it
/// (Enter does nothing), and so do the accessibility `Activate` and `Toggle`
/// actions. The look is the theme's
/// [`CheckboxTheme::root`](crate::theme::CheckboxTheme), given a
/// [`CheckableState`](crate::theme::CheckableState). `indeterminate` shows
/// and publishes the mixed (partially checked) state while true; pressing a
/// mixed check box writes `true`.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn terms() -> impl IntoView {
///     let agreed = signal(false);
///     column((
///         checkbox("I accept the terms", agreed),
///         text(move || if agreed.get() { "Accepted" } else { "Not accepted" }.to_owned()),
///     ))
///     .style(style().gap(8))
/// }
/// ```
#[component]
pub fn checkbox(
    label: Value<String>,
    checked: Signal<bool>,
    on_edit: Option<Callback<EditPhase>>,
    indeterminate: Option<Value<bool>>,
    size: Option<Value<ControlSize>>,
    hide_label: Option<Value<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let mut behavior = checkbox_behavior(checked, indeterminate);
    if let Some(observer) = on_edit {
        behavior = behavior.on_edit(move |phase| observer.call(phase));
    }
    // `.disabled(..)` ignores input while true.
    if let Some(disabled) = caller.disabled() {
        behavior = behavior.disabled(move || disabled.get());
    }
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let invalid = caller.invalid();
    let part = behavior
        .control()
        .map_state(move |state: ToggleState| CheckableState {
            input: state.input,
            control: CheckControl::CheckBox(state.checked),
            size: size.get(),
            invalid: invalid.as_ref().is_some_and(|invalid| invalid.get()),
        });
    let check_state = part.state();
    let indicator = crate::toggle::parts::checkbox(&variants, check_state);
    with_base_recipe_style(
        node().style(single_line_text()).name(label),
        |t: &CheckboxTheme| t.root_recipe(),
        &variants,
        move || check_state.get(),
    )
    .part(part)
    .child(visible_label(label, hide_label))
    .child(indicator)
}

/// A switch named and labelled `label` that edits `on`.
///
/// It has `Role::Switch` and the checked state; Space and click flip it
/// (Enter does nothing), and so do the accessibility `Activate` and `Toggle`
/// actions. The look is the theme's
/// [`SwitchTheme::root`](crate::theme::SwitchTheme).
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn notifications() -> impl IntoView {
///     let on = signal(true);
///     switch("Notifications", on)
/// }
/// ```
#[component]
pub fn switch(
    label: Value<String>,
    on: Signal<bool>,
    on_edit: Option<Callback<EditPhase>>,
    size: Option<Value<ControlSize>>,
    hide_label: Option<Value<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let mut behavior = use_switch(on);
    if let Some(observer) = on_edit {
        behavior = behavior.on_edit(move |phase| observer.call(phase));
    }
    // `.disabled(..)` ignores input while true.
    if let Some(disabled) = caller.disabled() {
        behavior = behavior.disabled(move || disabled.get());
    }
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let invalid = caller.invalid();
    let (control, drag) = switch_gesture(behavior.control(), on, size, on_edit);
    let kind = switch_control(on, drag);
    let part = control.map_state(move |state: ToggleState| CheckableState {
        input: state.input,
        control: kind(state.checked),
        size: size.get(),
        invalid: invalid.as_ref().is_some_and(|invalid| invalid.get()),
    });
    let check_state = part.state();
    let indicator = crate::toggle::parts::switch(&variants, check_state);
    with_base_recipe_style(
        node().style(single_line_text()).name(label),
        |t: &SwitchTheme| t.root_recipe(),
        &variants,
        move || check_state.get(),
    )
    .part(part)
    .child(visible_label(label, hide_label))
    .child(indicator)
}
