//! Role: The button group recipe state, tokens, recipe table and default looks.

use crate::control_size::ControlSize;
use crate::recipe::recipe_table;
use kivact_core::style::{Align, FlexDirection, style};
use kivact_core::view::Orientation;
pub(crate) fn button_root(
    theme: &kivact_core::style::Theme,
    look: &crate::theme::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let t = &theme.ext::<crate::UiTokens>().button_group;
    style()
        .flex_direction(if look.orientation == Orientation::Vertical {
            FlexDirection::Column
        } else if look.rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Stretch)
        .gap(t.gap)
        .border_radius(t.radius)
}

pub(crate) fn button_separator(
    theme: &kivact_core::style::Theme,
    look: &crate::theme::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let base = style()
        .flex_shrink(0)
        .align_self(Align::Stretch)
        .background(theme.ext::<crate::UiTokens>().button_group.divider);
    if look.orientation == Orientation::Vertical {
        base.height(1).margin_x(1)
    } else {
        base.width(1).margin_y(1)
    }
}

pub(crate) fn button_text(
    theme: &kivact_core::style::Theme,
    _: &crate::theme::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let t = &theme.ext::<crate::UiTokens>().button_group;
    style()
        .align_items(Align::Center)
        .gap(8)
        .padding_x(10)
        .border(1, t.divider)
        .background(t.text_background)
        .color(t.text_color)
        .font_size(12)
        .font_weight(500)
        .text_wrap(false)
}

pub(crate) fn button_member(
    theme: &kivact_core::style::Theme,
    look: &crate::theme::Look<ButtonGroupMemberState>,
) -> kivact_core::style::Style {
    let tokens = theme.ext::<crate::UiTokens>();
    let t = &tokens.button_group;
    if !t.joined {
        return style();
    }
    let first = if look.first { t.radius } else { 0.0 };
    let last = if look.last { t.radius } else { 0.0 };
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
}
