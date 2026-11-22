//! Role: Lyra recipes of the sidebar: the core sidebar's parts with the kit's
//! text sizes, row spacing and flat corners.
//!
//! The look of sidebar.tsx. The core draws the sidebar; these parts change
//! only the values where the Lyra look differs from the core's defaults. The
//! colors and the widths are the sidebar's theme variables, set by the kit's theme.

use crate::color::{SIDEBAR_FOREGROUND, SIDEBAR_RING};
use kivact::style::{Style, Theme, style};
use kivact::ui::theme::{Look, SidebarItemState, SidebarState, SidebarTheme};

/// The sidebar's theme: the menu buttons, sub buttons, group labels and the
/// header and footer sections take the Lyra values.
pub(crate) fn sidebar() -> SidebarTheme {
    SidebarTheme::default()
        .menu_button(menu_button)
        .sub_button(sub_button)
        .group_label(group_label)
        .header(section)
        .footer(section)
}

/// `text-xs p-2 gap-2 rounded-none`: a row with a 2 px ring on focus, medium
/// weight while it is the current page.
fn menu_button(_: &Theme, s: &Look<SidebarItemState>, base: Style) -> Style {
    let base = base
        .font_size(12.0)
        .padding(8.0)
        .gap(8.0)
        .border_radius(0.0);
    let base = if s.active {
        base.font_weight(500)
    } else {
        base
    };
    base.focus_visible(style().outline(2.0, SIDEBAR_RING))
}

/// `text-xs rounded-none`: a sub list's link.
fn sub_button(_: &Theme, _: &Look<SidebarItemState>, base: Style) -> Style {
    base.font_size(12.0).border_radius(0.0)
}

/// `h-8 px-2 text-xs`: the label of a group, in the sidebar's text at 70%.
fn group_label(_: &Theme, _: &Look<SidebarState>, base: Style) -> Style {
    base.font_size(12.0)
        .padding_x(8.0)
        .color(SIDEBAR_FOREGROUND.with_alpha(0.7))
}

/// `gap-2 p-2`: the header and the footer sections.
fn section(_: &Theme, _: &Look<SidebarState>, base: Style) -> Style {
    base.gap(8.0).padding(8.0)
}
