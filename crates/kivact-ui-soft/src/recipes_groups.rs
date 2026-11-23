//! Role: Install Astryx Neutral's recipes of the groups: joined buttons, toggle groups
//! and toolbars.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use crate::variants::{ButtonGroupVariant, SectionVariant, ToolbarVariant};
use kivact::style::{
    Align, Color, Display, FlexDirection, Justify, Length, Style, Theme, ThemeMode, Track, pct,
    style,
};
use kivact::ui::theme::{
    ButtonGroupMemberState, ButtonGroupState, ButtonGroupTheme, Look, ToggleButtonTheme,
    ToggleGroupItemState, ToggleGroupState, ToggleGroupTheme, ToolbarState, ToolbarTheme,
};
use kivact::ui::{ControlSize, ToolbarEdge};
use kivact::view::Orientation;

/// The flow of a group's items: a column when vertical, reversed in a right-to-left reading direction.
fn direction(orientation: Orientation, rtl: bool) -> FlexDirection {
    if orientation == Orientation::Vertical {
        FlexDirection::Column
    } else if rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    }
}

/// The surface of a joined button group: its items stretched in a row or column, with the group's elevation.
fn button_root(theme: &Theme, look: &Look<ButtonGroupState>, _base: Style) -> Style {
    let elevation = look.variant::<ButtonGroupVariant>().unwrap_or_default().0;
    shadows::elevate(
        style()
            .flex_direction(direction(look.orientation, look.rtl))
            .align_items(Align::Stretch)
            .border_radius(r::RADIUS_ELEMENT),
        elevation,
        theme.mode() == ThemeMode::Dark,
    )
}

/// A joined button: only the outer corners of the group are rounded.
fn button_member(theme: &Theme, look: &Look<ButtonGroupMemberState>, _base: Style) -> Style {
    let radius = theme.get(&r::RADIUS_ELEMENT_VAR);
    let first = if look.first { radius } else { 0.0 };
    let last = if look.last { radius } else { 0.0 };
    let corners = if look.layout.orientation == Orientation::Vertical {
        (first, first, last, last)
    } else if look.layout.rtl {
        (last, first, first, last)
    } else {
        (first, last, last, first)
    };
    style()
        .border_radius_corners(corners.0, corners.1, corners.2, corners.3)
        .clip(true)
}

/// The divider between joined buttons: a hairline across the group.
fn button_separator(_: &Theme, look: &Look<ButtonGroupState>, _base: Style) -> Style {
    let base = style()
        .flex_shrink(0.0)
        .align_self(Align::Stretch)
        .background(c::BORDER);
    if look.orientation == Orientation::Vertical {
        base.height(m::BORDER_WIDTH).margin_x(1)
    } else {
        base.width(m::BORDER_WIDTH).margin_y(1)
    }
}

/// The text of a group's auxiliary cell: the supporting role on the muted background, in one line.
fn button_text(_: &Theme, _: &Look<ButtonGroupState>, _base: Style) -> Style {
    style()
        .align_items(Align::Center)
        .gap(s::SPACE_S2)
        .padding_x(s::SPACE_S2)
        .border(m::BORDER_WIDTH, c::BORDER)
        .background(c::BG_MUTED)
        .color(c::TEXT_PRIMARY)
        .font_size(t::SUPPORTING.size_px)
        .font_weight(t::WEIGHT_MEDIUM)
        .single_line()
}

/// The row of a toggle group: its items with a gap that follows the group's spacing.
fn toggle_root(theme: &Theme, look: &Look<ToggleGroupState>, _base: Style) -> Style {
    style()
        .flex_direction(direction(look.orientation, look.rtl))
        .gap(
            look.spacing
                .map_or_else(|| theme.get(&s::SPACE_S1_VAR), |gap| gap * 4.0),
        )
        .align_items(if look.orientation == Orientation::Vertical {
            Align::Stretch
        } else {
            Align::Center
        })
}

/// A toggle of a group: the standard toggle button's recipe.
fn toggle_item(theme: &Theme, look: &Look<ToggleGroupItemState>, _base: Style) -> Style {
    theme
        .ext::<ToggleButtonTheme>()
        .root_recipe()
        .style(theme, &Look::new(look.toggle))
}

/// The surface of a toolbar: its fill, its padding, its height for the control size, and its dividers.
fn toolbar_root(theme: &Theme, look: &Look<ToolbarState>, _base: Style) -> Style {
    let fill = match look.variant::<ToolbarVariant>().unwrap_or_default().0 {
        SectionVariant::Section => c::BG_SURFACE,
        SectionVariant::Transparent => Color::TRANSPARENT,
        SectionVariant::Muted => c::BG_MUTED,
    };
    let height = match look.size {
        ControlSize::ExtraSmall | ControlSize::Small => m::CONTROL_SM,
        ControlSize::Medium => m::CONTROL_MD,
        ControlSize::Large => m::CONTROL_LG,
    };
    let padding = theme.get(&s::SPACE_S3_VAR);
    let vertical = look.orientation == Orientation::Vertical;
    let mut base = style()
        .background(fill)
        .padding(s::SPACE_S3)
        .width(pct(100))
        .min_height(height + padding * 2.0)
        .flex_direction(direction(look.orientation, look.rtl))
        .align_items(if vertical {
            Align::Stretch
        } else {
            Align::Center
        });
    if look.centered && !vertical {
        base = base.display(Display::Grid).grid_template_columns((
            Track::minmax(0, Track::fr(1)),
            Length::Auto,
            Track::minmax(0, Track::fr(1)),
        ));
    } else {
        base = base.justify_content(Justify::SpaceBetween);
    }
    for edge in &look.dividers {
        base = match edge {
            ToolbarEdge::Top => base.border_top(m::BORDER_WIDTH, c::BORDER),
            ToolbarEdge::Bottom => base.border_bottom(m::BORDER_WIDTH, c::BORDER),
            ToolbarEdge::Start if !look.rtl => base.border_left(m::BORDER_WIDTH, c::BORDER),
            ToolbarEdge::End if look.rtl => base.border_left(m::BORDER_WIDTH, c::BORDER),
            _ => base.border_right(m::BORDER_WIDTH, c::BORDER),
        };
    }
    base
}

/// Install the kit's recipes of the groups: joined buttons, toggle groups and toolbars.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme.with(
        ButtonGroupTheme::default()
            .root(button_root)
            .member(button_member)
            .separator(button_separator)
            .text(button_text),
    );
    let theme = theme.with(
        ToggleGroupTheme::default()
            .root(toggle_root)
            .item(toggle_item),
    );
    theme.with(ToolbarTheme::default().root(toolbar_root))
}
