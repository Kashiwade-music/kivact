//! Role: Build themed binary toggles from their actual interaction part state.
//!
//! A checkbox or a switch takes its value as the constructor's one
//! `Binding<bool>`, like a select and a text input; there is no hidden local
//! binding and no separate constant `checked` mode. Its recipe gets the real
//! checked state in a `CheckState`, and `.size` only reaches the recipe,
//! which decides the dimensions.
use super::layer::with_recipe_style;
use super::values::{CheckState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::control::{Node, NodeBuilder, node};
use kivact_core::primitive::NodeExt;
use kivact_core::view::{IntoText, IntoValue, IntoView, View};
use kivact_core::{Binding, Value};
use kivact_interaction::{ToggleState, use_checkbox, use_switch};
use std::cell::Cell;
use std::rc::Rc;
macro_rules! toggle_builder {
    ($(#[$doc:meta])* $name:ident, $kind:ident, $recipe:ident, $text:expr) => {
        $(#[$doc])*
        pub struct $name {
            root: Node,
            label: Value<String>,
            state: Binding<bool>,
            mixed: Option<Value<bool>>,
            part_state: Rc<Cell<Option<Value<CheckState>>>>,
            hide_label: Option<Value<bool>>,
            size: ControlSize,
            variants: Variants,
        }

        impl $name {
            fn create(label: Value<String>, state: Binding<bool>) -> Self {
                let part_state = Rc::new(Cell::new(None::<Value<CheckState>>));
                let read = part_state.clone();
                let variants = Variants::default();
                let root = with_recipe_style(
                    node().style(super::helpers::single_line_text()).name(label),
                    |recipes| recipes.$recipe(),
                    &variants,
                    move || read.get().expect("invariant: toggle part prepared before mount").get(),
                );
                Self {
                    root,
                    label,
                    state,
                    mixed: None,
                    part_state,
                    hide_label: None,
                    size: ControlSize::Medium,
                    variants,
                }
            }

            /// The size (default [`ControlSize::Medium`]), given to the
            /// recipe in its state; the recipe sizes the control by it.
            pub fn size(mut self, size: ControlSize) -> Self {
                self.size = size;
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
                let mut behavior = $kind(self.state);
                if let Some(mixed) = self.mixed {
                    behavior = behavior.indeterminate(mixed);
                }
                // `.disabled(..)` ignores input while true.
                if let Some(disabled) = self.root.accessible_disabled() {
                    behavior = behavior.disabled(move || disabled());
                }
                let invalid = self.root.accessible_invalid();
                let size = self.size;
                let text: Option<fn(Value<bool>) -> Value<String>> = $text;
                let text = text.map(|text| text(self.state.into_value()));
                let part = behavior.control().map_state(move |state: ToggleState| CheckState {
                    input: state.input,
                    checked: state.checked,
                    size,
                    invalid: invalid.as_ref().is_some_and(|invalid| invalid()),
                    text,
                });
                self.part_state.set(Some(part.state()));
                self.root
                    .part(part)
                    .child(super::helpers::visible_label(self.label, self.hide_label))
                    .into_view()
            }
        }
    };
}

toggle_builder!(
    /// A check box: a box and a label that flip a `bool` together.
    ///
    /// Created by [`checkbox`]. Its behavior and semantics are those of
    /// [`use_checkbox`].
    Checkbox,
    use_checkbox,
    checkbox_recipe,
    None
);

toggle_builder!(
    /// A switch: a track with a thumb and a label that flip a `bool`
    /// together.
    ///
    /// Created by [`switch`]. Its behavior and semantics are those of
    /// [`use_switch`].
    Switch,
    use_switch,
    switch_recipe,
    Some(switch_text)
);

/// A switch's localized word for its state, made at setup.
fn switch_text(on: Value<bool>) -> Value<String> {
    let (yes, no) = (
        kivact_core::i18n::message("switch.on", "ON"),
        kivact_core::i18n::message("switch.off", "OFF"),
    );
    kivact_core::memo(move || if on.get() { yes.get() } else { no.get() })
}

impl Checkbox {
    /// Show and publish the mixed (partially checked) state while true.
    /// The binding stays binary; pressing a mixed check box writes `true`.
    /// Clearing the mixed value is the caller's (see
    /// [`Toggle::indeterminate`](kivact_interaction::Toggle::indeterminate)).
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
/// [`CheckState`](super::CheckState).
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
pub fn checkbox(label: impl IntoText, checked: impl Into<Binding<bool>>) -> Checkbox {
    Checkbox::create(label.into_text_value(), checked.into())
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
pub fn switch(label: impl IntoText, on: impl Into<Binding<bool>>) -> Switch {
    Switch::create(label.into_text_value(), on.into())
}
