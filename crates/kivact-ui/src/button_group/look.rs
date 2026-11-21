//! Role: The button group recipe state, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use kivact_core::style::{Align, Color, FlexDirection, Length, Theme, ThemeVar, style};
use kivact_core::view::Orientation;

/// The gap between the buttons of a group.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::new("button_group.gap", 0.0);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);
/// The corner radius of a group, and of its joined edges.
pub static RADIUS_VAR: ThemeVar<f64> = ThemeVar::new("button_group.radius", 0.0);
/// [`RADIUS_VAR`] as a length for a style.
pub const RADIUS: Length = Length::var(&RADIUS_VAR);
/// The divider between the buttons of a group.
pub static DIVIDER_VAR: ThemeVar<Color> = ThemeVar::new("button_group.divider", Color::TRANSPARENT);
/// The divider between the buttons of a solid group.
pub static SOLID_DIVIDER_VAR: ThemeVar<Color> =
    ThemeVar::new("button_group.solid_divider", Color::TRANSPARENT);
/// The background of a group's text cell.
pub static TEXT_BACKGROUND_VAR: ThemeVar<Color> =
    ThemeVar::new("button_group.text_background", Color::TRANSPARENT);
/// The text color of a group's text cell.
pub static TEXT_COLOR_VAR: ThemeVar<Color> = ThemeVar::new("button_group.text_color", Color::BLACK);

pub(crate) fn button_root(
    _: &Theme,
    look: &crate::theme::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    style()
        .flex_direction(if look.orientation == Orientation::Vertical {
            FlexDirection::Column
        } else if look.rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Stretch)
        .gap(GAP)
        .border_radius(RADIUS)
}

pub(crate) fn button_separator(
    theme: &Theme,
    look: &crate::theme::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let base = style()
        .flex_shrink(0)
        .align_self(Align::Stretch)
        .background(theme.get(&DIVIDER_VAR));
    if look.orientation == Orientation::Vertical {
        base.height(1).margin_x(1)
    } else {
        base.width(1).margin_y(1)
    }
}

pub(crate) fn button_text(
    theme: &Theme,
    _: &crate::theme::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    style()
        .align_items(Align::Center)
        .gap(8)
        .padding_x(10)
        .border(1, theme.get(&DIVIDER_VAR))
        .background(theme.get(&TEXT_BACKGROUND_VAR))
        .color(theme.get(&TEXT_COLOR_VAR))
        .font_size(12)
        .font_weight(500)
        .text_wrap(false)
}

pub(crate) fn button_member(
    theme: &Theme,
    look: &crate::theme::Look<ButtonGroupMemberState>,
) -> kivact_core::style::Style {
    if !*theme.ext::<ButtonGroupTheme>().joined_value() {
        return style();
    }
    let radius = theme.get(&RADIUS_VAR);
    let first = if look.first { radius } else { 0.0 };
    let last = if look.last { radius } else { 0.0 };
    let (tl, tr, br, bl) = if look.layout.orientation == Orientation::Vertical {
        (first, first, last, last)
    } else if look.layout.rtl {
        (last, first, first, last)
    } else {
        (first, last, last, first)
    };
    style().border_radius_corners(tl, tr, br, bl).clip(true)
}

/// Layout supplied to group and auxiliary cell recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonGroupState {
    pub orientation: Orientation,
    pub size: ControlSize,
    pub rtl: bool,
}

/// Position of a child within a joined group, derived from the child list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonGroupMemberState {
    pub layout: ButtonGroupState,
    pub first: bool,
    pub last: bool,
}

recipe_table! {
    /// Joined action group surface and auxiliary cell recipes.
    ButtonGroupTheme {
        root(root_recipe): ButtonGroupState = button_root,
        member(member_recipe): ButtonGroupMemberState = button_member,
        separator(separator_recipe): ButtonGroupState = button_separator,
        text(text_recipe): ButtonGroupState = button_text,
    }
    settings {
        /// Whether the buttons of a group join their edges: a joined button has no radius of its own edges.
        joined(joined_value): bool = true,
    }
}
