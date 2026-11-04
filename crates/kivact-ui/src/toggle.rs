//! Role: Build themed binary toggles from their actual interaction part state.
//!
//! A checkbox or a switch takes its value as the constructor's one
//! `Binding<bool>`, like a select and a text input; there is no hidden local
//! binding and no separate constant `checked` mode. Its recipe gets the real
//! checked state in a `CheckableState`, and `.size` only reaches the recipe,
//! which decides the dimensions.
use super::layer::with_base_recipe_style;
use super::values::{CheckableState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::state::{Binding, Value};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::view::{IntoView, View};
use kivact_core::view::{Node, NodeBuilder, node};
use kivact_interaction::{CheckboxApi, SwitchApi, ToggleState, use_checkbox, use_switch};
macro_rules! toggle_builder {
    ($(#[$doc:meta])* $name:ident, $kind:ident, $recipe:ident, $parts:ident, $gesture:ident, $text:expr) => {
        $(#[$doc])*
        pub struct $name {
            root: Node,
            label: Value<String>,
            state: Binding<bool>,
            mixed: Option<Value<bool>>,
            hide_label: Option<Value<bool>>,
            size: Value<ControlSize>,
            variants: Variants,
        }

        impl $name {
            fn create(label: Value<String>, state: Binding<bool>) -> Self {
                Self {
                    root: node().style(super::helpers::single_line_text()).name(label),
                    label,
                    state,
                    mixed: None,
                    hide_label: None,
                    size: super::values::default_control_size().into_value(),
                    variants: Variants::default(),
                }
            }

            /// The size (default [`ControlSize::Medium`]), given to the
            /// recipe in its state; the recipe sizes the control by it.
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
            /// Give the component a [`Variant`]: a look beyond the meanings its
            /// state carries, which every recipe of the component reads with
            /// [`Look::variant`](crate::Look::variant). A later value of the same
            /// type replaces the earlier one; a theme that does not know the type
            /// draws the plain look.
            ///
            /// The value is a constant: the recipes read the values given before
            /// the component is mounted, and a value never changes afterwards. A
            /// look that changes while the component lives is a state the recipe
            /// reads, or a reactive `.style(..)` of the caller's.
            pub fn variant(self, v: impl Variant) -> Self {
                self.variants.add(v);
                self
            }
        }

        impl NodeBuilder for $name {
            fn node_mut(&mut self) -> &mut Node {
                &mut self.root
            }
        }

        #[diagnostic::do_not_recommend]
        impl IntoView for $name {
            fn into_view(mut self) -> View {
                let mut behavior = $kind(self.state, self.mixed);
                // `.disabled(..)` ignores input while true.
                if let Some(disabled) = self.root.accessible_disabled() {
                    behavior = behavior.disabled(move || disabled.get());
                }
                let invalid = self.root.accessible_invalid();
                let size = self.size;
                let text: Option<fn(Value<bool>) -> Value<String>> = $text;
                let text = text.map(|text| text(self.state.into_value()));
                let (control, drag) = $gesture(behavior.control(), self.state, size);
                let part = control.map_state(move |state: ToggleState| CheckableState {
                    input: state.input,
                    checked: state.checked,
                    size: size.get(),
                    invalid: invalid.as_ref().is_some_and(|invalid| invalid.get()),
                    text,
                    switch_drag: drag.and_then(|drag| drag.get()),
                });
                let check_state = part.state();
                let indicator = super::toggle_parts::$parts(&self.variants, check_state);
                with_base_recipe_style(
                    self.root,
                    |recipes| recipes.$recipe(),
                    &self.variants,
                    move || check_state.get(),
                )
                    .part(part)
                    .child(super::helpers::visible_label(self.label, self.hide_label))
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
    checkbox_recipe,
    checkbox,
    no_gesture,
    None
);

toggle_builder!(
    /// A switch: a track with a thumb and a label that flip a `bool`
    /// together.
    ///
    /// Created by [`switch`]. Its behavior and semantics are those of
    /// [`use_switch`](kivact_interaction::use_switch).
    Switch,
    switch_behavior,
    switch_recipe,
    switch,
    switch_gesture,
    Some(switch_text)
);

use super::switch_drag::switch as switch_gesture;

fn no_gesture(
    control: kivact_core::custom::Part<ToggleState>,
    _: Binding<bool>,
    _: Value<ControlSize>,
) -> (
    kivact_core::custom::Part<ToggleState>,
    super::switch_drag::Visual,
) {
    (control, None)
}

/// The behavior of a check box, with its mixed state when it has one.
fn checkbox_behavior(state: Binding<bool>, mixed: Option<Value<bool>>) -> CheckboxApi {
    let behavior = use_checkbox(state);
    match mixed {
        Some(mixed) => behavior.indeterminate(mixed),
        None => behavior,
    }
}

/// The behavior of a switch; a switch has no mixed state.
fn switch_behavior(state: Binding<bool>, _mixed: Option<Value<bool>>) -> SwitchApi {
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

impl Checkbox {
    /// Show and publish the mixed (partially checked) state while true.
    /// The binding stays binary; pressing a mixed check box writes `true`.
    /// Clearing the mixed value is the caller's (see
    /// [`CheckboxApi::indeterminate`](kivact_interaction::CheckboxApi::indeterminate)).
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
/// [`UiRecipes::checkbox`](super::UiRecipes), given a
/// [`CheckableState`](super::CheckableState).
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
pub fn checkbox(
    label: impl IntoText,
    checked: impl kivact_core::state::IntoBinding<bool>,
) -> Checkbox {
    Checkbox::create(label.into_text_value(), checked.into_binding())
}

/// A switch named and labelled `label` that edits `on`.
///
/// It has `Role::Switch` and the checked state; Space and click flip it
/// (Enter does nothing), and so do the accessibility `Activate` and `Toggle`
/// actions. The look is the theme's
/// [`UiRecipes::switch`](super::UiRecipes).
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
pub fn switch(label: impl IntoText, on: impl kivact_core::state::IntoBinding<bool>) -> Switch {
    Switch::create(label.into_text_value(), on.into_binding())
}
