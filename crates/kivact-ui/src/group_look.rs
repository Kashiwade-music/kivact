//! Role: Default recipes for passive action group layout snapshots.
use crate::group_types::{
    ButtonGroupMemberState, ButtonGroupState, ToggleGroupItemState, ToggleGroupState,
};
use kivact_core::style::{Align, Direction, style};
use kivact_core::view::Orientation;
pub(crate) fn button_root(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let t = &theme.ext::<crate::UiTokens>().button_group;
    style()
        .direction(if look.orientation == Orientation::Vertical {
            Direction::Column
        } else if look.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Stretch)
        .gap(t.gap)
        .radius(t.radius)
}
pub(crate) fn button_separator(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ButtonGroupState>,
) -> kivact_core::style::Style {
    let base = style()
        .shrink(0)
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
    _: &crate::Look<ButtonGroupState>,
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
        .weight(500)
        .text_wrap(false)
}

pub(crate) fn button_member(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ButtonGroupMemberState>,
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
    style().radius_corners(tl, tr, br, bl).clip(true)
}
pub(crate) fn toggle_root(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ToggleGroupState>,
) -> kivact_core::style::Style {
    style()
        .direction(if look.orientation == Orientation::Vertical {
            Direction::Column
        } else if look.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Stretch)
        .gap(
            look.spacing
                .map_or(theme.ext::<crate::UiTokens>().toggle_group.gap, |spacing| {
                    spacing * 4.0
                }),
        )
}
pub(crate) fn toggle_item(
    theme: &kivact_core::style::Theme,
    look: &crate::Look<ToggleGroupItemState>,
) -> kivact_core::style::Style {
    crate::look::toggle_button(theme, &crate::Look::new(look.toggle))
}
