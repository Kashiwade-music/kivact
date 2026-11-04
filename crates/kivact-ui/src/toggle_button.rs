//! Role: Build the themed toggle_button button from its actual interaction part state.
//!
//! A toggle_button takes its value as the constructor's one `Binding<bool>`, like a
//! checkbox. Its recipe gets the real pressed state in a [`ToggleButtonState`];
//! the `.icon` view is a child of the root, and the recipe lays the row out.
//! The recipe is the root's base style, so the caller's `.style(..)` wins
//! whatever the order of the calls.
use super::layer::with_base_recipe_style;
use super::values::{ControlSize, ToggleButtonState};
use super::variants::{Variant, Variants};
use kivact_core::custom::{Constraints, primitive};
use kivact_core::draw::TextFormat;
use kivact_core::geometry::Size;
use kivact_core::state::Binding;
use kivact_core::state::Value;
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::style::{Align, TextAlign, Theme, ThemeExtension, style, use_theme};
use kivact_core::view::{IntoView, View, dynamic, show};
use kivact_core::view::{Node, NodeBuilder, column, node, text};
use kivact_interaction::{ToggleState, use_toggle_button};

/// Opt-in text measurement for a toggle_button whose label changes weight when pressed.
/// Install it with [`Theme::with`] and use the same size as the toggle_button recipe.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ToggleLabelReservation {
    /// Let the visible label determine its own width.
    #[default]
    Natural,
    /// Reserve space for the same label shaped at another weight.
    Reserve {
        /// The weight used in the pressed appearance.
        weight: u16,
        /// The recipe's font size in logical pixels.
        size: f64,
        /// The recipe's font size at [`ControlSize::ExtraSmall`].
        extra_small_size: f64,
    },
}

impl ThemeExtension for ToggleLabelReservation {
    fn fallback(_: &Theme) -> Self {
        Self::Natural
    }
}

fn toggle_label(label: Value<String>, hidden: Option<Value<bool>>, size: ControlSize) -> View {
    dynamic(
        || *use_theme().get().ext::<ToggleLabelReservation>(),
        move |reservation| {
            let ToggleLabelReservation::Reserve {
                weight,
                size: font_size,
                extra_small_size,
            } = reservation
            else {
                return super::helpers::visible_label(label, hidden);
            };
            let font_size = if size == ControlSize::ExtraSmall {
                extra_small_size
            } else {
                font_size
            };
            let visible = move || {
                let reservation = primitive()
                    .measure(
                        label,
                        move |label: &String, constraints: Constraints<'_>| {
                            let text = TextFormat {
                                weight,
                                size: font_size,
                                ..TextFormat::default()
                            };
                            let width = constraints.measure_text(label, &text).size.width;
                            Size::new(width, 0.0)
                        },
                    )
                    .style(style().min_width(0).height(0))
                    .hit_testable(false);
                column((
                    reservation,
                    text(label).style(super::helpers::label_style().text_align(TextAlign::Center)),
                ))
                .style(style().min_width(0).align_items(Align::Stretch))
            };
            match hidden {
                Some(hidden) => show(move || !hidden.get(), visible).into_view(),
                None => visible().into_view(),
            }
        },
    )
}

/// A button that stays pressed, created by [`toggle_button`]. Its behavior and
/// semantics are those of [`use_toggle_button`].
pub struct ToggleButton {
    root: Node,
    label: Value<String>,
    pressed: Binding<bool>,
    icon: Option<View>,
    hide_label: Option<Value<bool>>,
    size: ControlSize,
    variants: Variants,
}

/// A toggle_button button named and labelled `label` that edits `pressed`.
///
/// It has `Role::Button` with a checked state (ARIA's `aria-pressed`); Space,
/// Enter and click flip it, and so do the accessibility `Activate` and
/// `ToggleButton` actions. The look is the theme's
/// [`UiRecipes::toggle_button`](super::UiRecipes), given a
/// [`ToggleButtonState`](super::ToggleButtonState). A toggle_button with an
/// [`icon`](ToggleButton::icon) and a hidden label is an icon toggle_button, a square.
///
/// ```no_run
/// use kivact::view::icon;
/// use kivact::prelude::*;
/// use kivact::ui::toggle_button;
///
/// fn bold() -> impl IntoView {
///     let on = signal(false);
///     toggle_button("Bold", on).icon(icon("bold")).hide_label(true)
/// }
/// ```
#[track_caller]
pub fn toggle_button(
    label: impl IntoText,
    pressed: impl kivact_core::state::IntoBinding<bool>,
) -> ToggleButton {
    let label = label.into_text_value();
    ToggleButton {
        root: node().name(label),
        label,
        pressed: pressed.into_binding(),
        icon: None,
        hide_label: None,
        size: super::values::default_control_size(),
        variants: Variants::default(),
    }
}

impl ToggleButton {
    /// The size (default [`ControlSize::Medium`]), given to the recipe in
    /// its state; the recipe sizes the toggle_button by it.
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
    /// true; constant or reactive). With an [`icon`](Self::icon) the toggle_button
    /// is an icon toggle_button: the recipe knows it from
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

impl NodeBuilder for ToggleButton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[diagnostic::do_not_recommend]
impl IntoView for ToggleButton {
    fn into_view(mut self) -> View {
        let mut behavior = use_toggle_button(self.pressed);
        // `.disabled(..)` ignores input while true.
        if let Some(disabled) = self.root.accessible_disabled() {
            behavior = behavior.disabled(move || disabled.get());
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
        .child(toggle_label(self.label, self.hide_label, size))
        .into_view()
    }
}
