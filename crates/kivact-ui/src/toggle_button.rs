//! Role: Build the themed toggle button from its actual interaction part state.
//!
//! A toggle takes its value as the constructor's one `Binding<bool>`, like a
//! checkbox. Its recipe gets the real pressed state in a [`ToggleButtonState`];
//! the `.icon` view is a child of the root, and the recipe lays the row out.
//! The recipe is the root's base style, so the caller's `.style(..)` wins
//! whatever the order of the calls.
use super::layer::with_base_recipe_style;
use super::values::{ControlSize, ToggleButtonState};
use super::variants::{Variant, Variants};
use kivact_core::Binding;
use kivact_core::Value;
use kivact_core::control::{Node, NodeBuilder, node};
use kivact_core::primitive::NodeExt;
use kivact_core::view::{IntoText, IntoValue, IntoView, View};
use kivact_interaction::{ToggleState, use_toggle_button};

/// A button that stays pressed, created by [`toggle`]. Its behavior and
/// semantics are those of [`use_toggle_button`].
pub struct Toggle {
    root: Node,
    label: Value<String>,
    pressed: Binding<bool>,
    icon: Option<View>,
    hide_label: Option<Value<bool>>,
    size: ControlSize,
    variants: Variants,
}

/// A toggle button named and labelled `label` that edits `pressed`.
///
/// It has `Role::Button` with a checked state (ARIA's `aria-pressed`); Space,
/// Enter and click flip it, and so do the accessibility `Activate` and
/// `Toggle` actions. The look is the theme's
/// [`UiRecipes::toggle`](super::UiRecipes), given a
/// [`ToggleButtonState`](super::ToggleButtonState). A toggle with an
/// [`icon`](Toggle::icon) and a hidden label is an icon toggle, a square.
///
/// ```no_run
/// use kivact::control::icon;
/// use kivact::prelude::*;
/// use kivact::ui::toggle;
///
/// fn bold() -> impl IntoView {
///     let on = signal(false);
///     toggle("Bold", on).icon(icon("bold")).hide_label(true)
/// }
/// ```
#[track_caller]
pub fn toggle(label: impl IntoText, pressed: impl Into<Binding<bool>>) -> Toggle {
    let label = label.into_text_value();
    Toggle {
        root: node().name(label),
        label,
        pressed: pressed.into(),
        icon: None,
        hide_label: None,
        size: ControlSize::Medium,
        variants: Variants::default(),
    }
}

impl Toggle {
    /// The size (default [`ControlSize::Medium`]), given to the recipe in
    /// its state; the recipe sizes the toggle by it.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// A view before the label; the recipe knows it from
    /// [`ToggleButtonState::leading_icon`].
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Keep the accessible name while omitting the visible label (while
    /// true; constant or reactive). With an [`icon`](Self::icon) the toggle
    /// is an icon toggle: the recipe knows it from
    /// [`ToggleButtonState::icon_only`].
    #[track_caller]
    pub fn hide_label(mut self, hide: impl IntoValue<bool>) -> Self {
        self.hide_label = Some(hide.into_value());
        self
    }

    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which the recipe reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipe reads the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}

impl NodeBuilder for Toggle {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[diagnostic::do_not_recommend]
impl IntoView for Toggle {
    fn into_view(mut self) -> View {
        let mut behavior = use_toggle_button(self.pressed);
        // `.disabled(..)` ignores input while true.
        if let Some(disabled) = self.root.accessible_disabled() {
            behavior = behavior.disabled(move || disabled());
        }
        let (size, hide) = (self.size, self.hide_label);
        let leading_icon = self.icon.is_some();
        let part = behavior
            .control()
            .map_state(move |state: ToggleState| ToggleButtonState {
                input: state.input,
                pressed: state.checked.is_checked(),
                size,
                leading_icon,
                icon_only: leading_icon && hide.is_some_and(|hide| hide.get()),
            });
        let state = part.state();
        let root = self.root.base_style(super::helpers::single_line_text());
        with_base_recipe_style(
            root,
            |recipes| recipes.toggle_recipe(),
            &self.variants,
            move || state.get(),
        )
        .part(part)
        .child(self.icon)
        .child(super::helpers::visible_label(self.label, self.hide_label))
        .into_view()
    }
}
