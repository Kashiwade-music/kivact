//! Role: Style Astryx Neutral's command palette: its scrim and panel, the search field,
//! the grouped rows and the footer.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::{body_text, modal, scrim, shadowed, supporting_text};
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, PopupState};
use kivact::style::{
    Align, Color, FlexDirection, Overflow, Style, TextAlign, Theme, ThemeMode, auto, pct, style,
};
use kivact::ui::theme::{CommandPaletteTheme, Look};

const PALETTE_WIDTH: f64 = 640.0;
const PALETTE_MAX_HEIGHT: f64 = 480.0;

/// The layer behind the palette: the blurred scrim.
fn palette_scrim(theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    scrim(theme)
}

/// The modal panel, which holds the search, the list and the footer.
fn panel(theme: &Theme, _look: &Look<PopupState>, _base: Style) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    let gutter = theme.get(&s::SPACE_LAYER_VIEWPORT_GUTTER_VAR);
    modal(
        shadowed(body_text(style()), shadows::high(dark))
            .background(c::BG_SURFACE)
            .border_radius(r::RADIUS_CONTAINER)
            .flex_direction(FlexDirection::Column)
            .width(PALETTE_WIDTH)
            .max_width(pct(100) - gutter * 2.0)
            .max_height(PALETTE_MAX_HEIGHT)
            .overflow(Overflow::Hidden),
        theme,
    )
}

/// The row of the search field, under a rule.
fn search(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .flex_shrink(0)
        .padding_x(s::SPACE_S4)
        .padding_y(s::SPACE_S3)
        .gap(s::SPACE_S2)
        .border_bottom(m::BORDER_WIDTH, c::BORDER)
}

/// The magnifier before the field.
fn search_icon(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    icon_box()
}

/// The search field: a native editing leaf with no frame.
fn field(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    body_text(style())
        .flex_grow(1)
        .min_width(0)
        .height(t::BODY.line_height_px)
        .background(Color::TRANSPARENT)
        .placeholder_color(c::TEXT_SECONDARY)
        .caret_color(c::TEXT_PRIMARY)
}

/// The scrolling box of the groups and rows.
fn list(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .min_height(0)
        .padding(s::SPACE_S1)
        .gap(s::SPACE_S0_5)
}

/// The heading of a group.
fn group(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    supporting_text(style())
        .padding_x(s::SPACE_S3)
        .padding_y(s::SPACE_S1)
}

/// A command's row: highlighted while it is active, tinted while it is selected.
fn row(_theme: &Theme, look: &Look<ListItemState>, _base: Style) -> Style {
    let fill = if look.active {
        c::OVERLAY_HOVER
    } else if look.selected {
        c::ACCENT_MUTED
    } else {
        Color::TRANSPARENT
    };
    let base = body_text(style())
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(s::SPACE_S2)
        .padding_x(s::SPACE_S3)
        .padding_y(s::SPACE_S2)
        .min_height(m::CONTROL_LG)
        .border_radius(r::RADIUS_INNER)
        .background(fill);
    if look.input.is_disabled() {
        base.opacity(m::DISABLED_OPACITY)
    } else {
        base.hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED))
    }
}

/// The icon at the start of a row.
fn icon(_theme: &Theme, _look: &Look<ListItemState>, _base: Style) -> Style {
    icon_box()
}

/// The keyboard hint at the end of a row.
fn hint(_theme: &Theme, _look: &Look<ListItemState>, _base: Style) -> Style {
    supporting_text(style()).margin_left(auto()).flex_shrink(0)
}

/// The text shown when no command matches: centered in the list.
fn empty(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    supporting_text(style())
        .padding_x(s::SPACE_S4)
        .padding_y(s::SPACE_S8)
        .align_self(Align::Stretch)
        .text_align(TextAlign::Center)
}

/// The footer under the list, over a rule.
fn footer(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    supporting_text(style())
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(s::SPACE_S4)
        .padding_x(s::SPACE_S4)
        .padding_y(s::SPACE_S2)
        .border_top(m::BORDER_WIDTH, c::BORDER)
        .flex_shrink(0)
}

/// A square icon box in the secondary text color.
fn icon_box() -> Style {
    style()
        .size(m::ICON_SM, m::ICON_SM)
        .flex_shrink(0)
        .color(c::TEXT_SECONDARY)
}

/// Install the kit's recipes of the command palette.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CommandPaletteTheme::default()
            .scrim(palette_scrim)
            .panel(panel)
            .search(search)
            .search_icon(search_icon)
            .field(field)
            .list(list)
            .group(group)
            .row(row)
            .icon(icon)
            .hint(hint)
            .empty(empty)
            .footer(footer),
    )
}
