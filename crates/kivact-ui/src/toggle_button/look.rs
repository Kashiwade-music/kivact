//! Role: The toggle button recipe state, tokens, recipe table and default looks.
use crate::look_common::{font_size, gap, height};
use crate::recipe::recipe_table;
use crate::theme::Look;

use crate::control_size::ControlSize;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, Color, FlexDirection, Justify, Style, Theme, style};
/// The toggle button recipe: a row of an optional icon and the label,
/// filled with the theme's pressed color while it is on.
pub(crate) fn toggle_button(theme: &Theme, state: &Look<ToggleButtonState>) -> Style {
    let size = state.size;
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap(theme, size))
        .height(height(theme, size))
        .min_width(height(theme, size))
        .padding_x(match size {
            ControlSize::ExtraSmall => theme.get(&kivact_core::style::space::SM_VAR),
            ControlSize::Small => theme.get(&kivact_core::style::space::MD_VAR),
            ControlSize::Medium | ControlSize::Large => {
                theme.get(&kivact_core::style::space::LG_VAR)
            }
        })
        .border_radius(theme.get(&kivact_core::style::radius::MD_VAR))
        .font_size(font_size(theme, size))
        .font_weight(theme.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
        .border(1, Color::TRANSPARENT)
        .color(kivact_core::style::color::TEXT)
        .hover(style().background(kivact_core::style::color::HOVER))
        .active(style().background(kivact_core::style::color::PRESSED));
    let s = if state.is_icon_only() {
        s.width(height(theme, size)).padding_x(0)
    } else {
        s
    };
    if state.pressed {
        s.background(kivact_core::style::color::PRESSED)
            .border_color(kivact_core::style::color::BORDER)
    } else {
        s
    }
}

/// The state of a [`toggle_button`](super::toggle) for its recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleButtonState {
    /// The actual input state of the toggle.
    pub input: PartState,
    /// Whether it is pressed (on).
    pub pressed: bool,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    /// Whether the toggle shows a leading icon ([`ToggleButton::icon`](super::ToggleButton::icon)).
    pub leading_icon: bool,
    /// Whether the label is hidden ([`ToggleButton::hide_label`](super::ToggleButton::hide_label),
    /// or [`ToggleGroup::icon_only`](super::ToggleGroup::icon_only) for a group's item).
    pub label_hidden: bool,
}

impl ToggleButtonState {
    /// Whether only the leading icon shows: the toggle has one and its label
    /// is hidden. A look draws it as a square of the toggle's height.
    pub fn is_icon_only(&self) -> bool {
        self.leading_icon && self.label_hidden
    }
}

recipe_table! {
    /// The recipes of a toggle button.
    ToggleButtonTheme {
        root(root_recipe): ToggleButtonState = toggle_button,
    }
}
