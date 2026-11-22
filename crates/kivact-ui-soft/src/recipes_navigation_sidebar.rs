//! Role: Install Astryx Neutral's sidebar recipes: the group labels, the menu rows and
//! their icons, the content and the footer.
use crate::color as c;
use crate::metrics as m;
use crate::recipe_parts::ring;
use crate::space::{self as s, Space};
use crate::type_scale as t;
use kivact::style::{Align, Color, FlexDirection, Justify, Style, Theme, auto, style};
use kivact::ui::SidebarMenuButtonSize;
use kivact::ui::theme::{Look, SidebarItemState, SidebarState, SidebarTheme};

/// The height of a menu row: the kit's control heights for its size.
fn row_height(size: SidebarMenuButtonSize) -> f64 {
    match size {
        SidebarMenuButtonSize::Sm => m::CONTROL_SM,
        SidebarMenuButtonSize::Default => m::CONTROL_MD,
        SidebarMenuButtonSize::Lg => m::CONTROL_LG,
    }
}

/// The horizontal padding of a menu row: 4 px in a small row, 8 px otherwise.
fn row_padding(size: SidebarMenuButtonSize) -> f64 {
    if size == SidebarMenuButtonSize::Sm {
        Space::S1.px()
    } else {
        Space::S2.px()
    }
}

/// The content: the menu rows' gap and the padding around them.
fn content(_theme: &Theme, _look: &Look<SidebarState>, base: Style) -> Style {
    base.gap(s::SPACE_S0_5)
        .padding_x(s::SPACE_S2)
        .padding_y(s::SPACE_S1)
}

/// The footer: pushed to the bottom of the panel, with a small gap above it.
fn footer(_theme: &Theme, _look: &Look<SidebarState>, base: Style) -> Style {
    base.margin_top(auto()).padding_top(s::SPACE_S1)
}

/// A group's label: a small semibold line in the secondary text color.
fn group_label(_theme: &Theme, _look: &Look<SidebarState>, base: Style) -> Style {
    base.font_size(t::SUPPORTING.size_px)
        .line_height(t::SUPPORTING.line_height_px / t::SUPPORTING.size_px)
        .font_weight(t::WEIGHT_SEMIBOLD)
        .color(c::TEXT_SECONDARY)
        .padding_x(s::SPACE_S2)
        .padding_y(s::SPACE_S1)
}

/// A group's description: a supporting line in the secondary text color.
fn group_description(_theme: &Theme, _look: &Look<SidebarState>, base: Style) -> Style {
    base.font_size(t::SUPPORTING.size_px)
        .line_height(t::SUPPORTING.line_height_px / t::SUPPORTING.size_px)
        .color(c::TEXT_SECONDARY)
}

/// A menu row: its control height and label role, with the neutral fill while it is the current
/// page. An unselected row shows the overlay on hover and press.
fn menu_button(_theme: &Theme, look: &Look<SidebarItemState>, base: Style) -> Style {
    let row = base
        .height(row_height(look.size))
        .padding_x(row_padding(look.size))
        .padding_y(0)
        .font_size(t::LABEL.size_px)
        .line_height(t::LABEL.line_height_px / t::LABEL.size_px)
        .font_weight(if look.active {
            t::WEIGHT_MEDIUM
        } else {
            t::WEIGHT_NORMAL
        })
        .color(c::TEXT_PRIMARY)
        .focus_visible(ring(c::ACCENT));
    if look.active {
        row.background(c::NEUTRAL)
            .hover(style().background(c::NEUTRAL))
            .active(style().background(c::NEUTRAL))
    } else {
        row.background(Color::TRANSPARENT)
            .hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED))
    }
}

/// A menu row's icon: an icon box, in the text color while the row is the current page.
fn menu_button_icon(_theme: &Theme, look: &Look<SidebarItemState>, base: Style) -> Style {
    base.size(m::ICON_SM, m::ICON_SM)
        .flex_shrink(0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(if look.active {
            c::TEXT_PRIMARY
        } else {
            c::ICON_SECONDARY
        })
}

/// A menu row's chevron: a small icon in the secondary icon color.
fn menu_button_chevron(_theme: &Theme, _look: &Look<SidebarItemState>, base: Style) -> Style {
    base.size(m::ICON_SM, m::ICON_SM)
        .flex_direction(FlexDirection::Row)
        .color(c::ICON_SECONDARY)
}

/// The sidebar's recipes for the kit.
pub(crate) fn sidebar() -> SidebarTheme {
    SidebarTheme::default()
        .content(content)
        .footer(footer)
        .group_label(group_label)
        .group_description(group_description)
        .menu_button(menu_button)
        .menu_button_icon(menu_button_icon)
        .menu_button_chevron(menu_button_chevron)
}
