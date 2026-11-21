//! Role: The checkbox and switch builders, their rendering and their gestures.

use crate::toggle::switch_drag::switch as switch_gesture;

use crate::control_size::ControlSize;
use crate::layer::with_base_recipe_style;
use crate::toggle::look::CheckControl;
use crate::toggle::look::{CheckableState, CheckboxTheme, SwitchTheme};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::state::{Signal, Value};
use kivact_core::view::{CheckedState, Element, Node, node};
use kivact_core::view::{IntoView, View};
use kivact_interaction::{CheckboxApi, SwitchApi, ToggleState, use_checkbox, use_switch};
macro_rules! toggle_builder {
    ($(#[$doc:meta])* $name:ident, $kind:ident, $table:ident, $parts:ident, $gesture:ident, $control:ident) => {
        $(#[$doc])*
        pub struct $name {
            root: Node,
            label: Value<String>,
            state: Signal<bool>,
            on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
            mixed: Option<Value<bool>>,
            hide_label: Option<Value<bool>>,
            size: Value<ControlSize>,
        }

        impl $name {
            /// Observe the beginning and commit of each value edit.
            pub fn on_edit<R: kivact_core::view::HandlerResult>(
                mut self,
                f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
            ) -> Self {
                self.on_edit = Some(kivact_core::state::callback(f));
                self
            }

            fn create(label: Value<String>, state: Signal<bool>) -> Self {
                Self {
                    root: node().style(crate::helpers::single_line_text()).name(label),
                    label,
                    state,
                    mixed: None,
                    on_edit: None,
                    hide_label: None,
                    size: crate::control_size::default_control_size().into_value(),
                }
            }

            /// The size (default [`ControlSize::Medium`]), given to the
            /// recipe in its state; the recipe sizes the control by it.
            #[track_caller]
            pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
                self.size = size.into_value();
                self
            }

            /// Keep the accessible name while omitting the visible label
            /// (while true; constant or reactive).
            #[track_caller]
            pub fn hide_label(mut self, hide: impl IntoValue<bool>) -> Self {
                self.hide_label = Some(hide.into_value());
                self
            }
        }

        impl $name {
        }

        impl Element for $name {
            fn node_mut(&mut self) -> &mut Node {
                &mut self.root
            }
        }

        #[diagnostic::do_not_recommend]
        impl IntoView for $name {
            fn into_view(mut self) -> View {
                let variants = self.root.authored().variants();
                let mut behavior = $kind(self.state, self.mixed);
                if let Some(observer) = self.on_edit {
                    behavior = behavior.on_edit(move |phase| observer.call(phase));
                }
                // `.disabled(..)` ignores input while true.
                if let Some(disabled) = self.root.accessible_disabled() {
                    behavior = behavior.disabled(move || disabled.get());
                }
                let invalid = self.root.accessible_invalid();
                let size = self.size;
                let (control, drag) =
                    $gesture(behavior.control(), self.state, size, self.on_edit);
                let kind = $control(self.state, drag);
                let part = control.map_state(move |state: ToggleState| CheckableState {
                    input: state.input,
                    control: kind(state.checked),
                    size: size.get(),
                    invalid: invalid.as_ref().is_some_and(|invalid| invalid.get()),
                });
                let check_state = part.state();
                let indicator = crate::toggle::parts::$parts(&variants, check_state);
                with_base_recipe_style(
                    self.root,
                    |t: &$table| t.root_recipe(),
                    &variants,
                    move || check_state.get(),
                )
                    .part(part)
                    .child(crate::helpers::visible_label(self.label, self.hide_label))
                    .child(indicator)
                    .into_view()
            }
        }
    };
}

toggle_builder!(
    /// A check box: a box and a label that flip a `bool` together.
    ///
    /// Created by [`checkbox`]. Its behavior and semantics are those of
    /// [`use_checkbox`](kivact_interaction::use_checkbox).
    Checkbox,
    checkbox_behavior,
    CheckboxTheme,
    checkbox,
    no_gesture,
    checkbox_control
);

toggle_builder!(
    /// A switch: a track with a thumb and a label that flip a `bool`
    /// together.
    ///
    /// Created by [`switch`]. Its behavior and semantics are those of
    /// [`use_switch`](kivact_interaction::use_switch).
    Switch,
    switch_behavior,
    SwitchTheme,
    switch,
    switch_gesture,
    switch_control
);

fn no_gesture(
    control: kivact_core::custom::Part<ToggleState>,
    _: Signal<bool>,
    _: Value<ControlSize>,
    _: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
) -> (
    kivact_core::custom::Part<ToggleState>,
    crate::toggle::switch_drag::Visual,
) {
    (control, None)
}

/// The behavior of a check box, with its mixed state when it has one.
fn checkbox_behavior(state: Signal<bool>, mixed: Option<Value<bool>>) -> CheckboxApi {
    let behavior = use_checkbox(state);
    match mixed {
        Some(mixed) => behavior.indeterminate(mixed),
        None => behavior,
    }
}

/// The behavior of a switch; a switch has no mixed state.
fn switch_behavior(state: Signal<bool>, _mixed: Option<Value<bool>>) -> SwitchApi {
    use_switch(state)
}

/// A switch's localized word for its state, made at setup.
fn switch_text(on: Value<bool>) -> Value<String> {
    let (yes, no) = (
        kivact_core::i18n::message("switch.on", "ON"),
        kivact_core::i18n::message("switch.off", "OFF"),
    );
    kivact_core::state::memo(move || if on.get() { yes.get() } else { no.get() })
}

/// A check box's control: its checked state as the behavior reports it,
/// mixed included.
fn checkbox_control(
    _: Signal<bool>,
    _: crate::toggle::switch_drag::Visual,
) -> impl Fn(CheckedState) -> CheckControl + 'static {
    CheckControl::CheckBox
}

/// A switch's control: on or off, its localized word for the state (made at
/// setup) and the live visual of its opted-in drag.
fn switch_control(
    state: Signal<bool>,
    drag: crate::toggle::switch_drag::Visual,
) -> impl Fn(CheckedState) -> CheckControl + 'static {
    let text = switch_text(state.into_value());
    move |checked| CheckControl::Switch {
        on: checked.is_checked(),
        text,
        drag: drag.and_then(|drag| drag.get()),
    }
}

impl Checkbox {
    /// Show and publish the mixed (partially checked) state while true.
    /// The binding stays binary; pressing a mixed check box writes `true`.
    /// Clearing the mixed value is the caller's (see
    /// [`CheckboxApi::indeterminate`](kivact_interaction::CheckboxApi::indeterminate)).
    #[track_caller]
    pub fn indeterminate(mut self, mixed: impl IntoValue<bool>) -> Self {
        self.mixed = Some(mixed.into_value());
        self
    }
}

/// A check box named and labelled `label` that edits `checked`.
///
/// It has `Role::CheckBox` and the checked state; Space and click flip it
/// (Enter does nothing), and so do the accessibility `Activate` and `Toggle`
/// actions. The look is the theme's
/// [`CheckboxTheme::root`](crate::theme::CheckboxTheme), given a
/// [`CheckableState`](crate::theme::CheckableState).
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
#[track_caller]
pub fn checkbox(label: impl IntoText, checked: kivact_core::state::Signal<bool>) -> Checkbox {
    Checkbox::create(label.into_text_value(), checked)
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
#[track_caller]
pub fn switch(label: impl IntoText, on: kivact_core::state::Signal<bool>) -> Switch {
    Switch::create(label.into_text_value(), on)
}
