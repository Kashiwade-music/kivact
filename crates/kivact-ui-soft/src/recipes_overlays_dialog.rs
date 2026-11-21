//! Role: Style Astryx Neutral's dialogs: the modal, alert, drawer and popover kinds, with their scrim and parts.
use crate::color as c;
use crate::metrics as m;
use crate::motion::MEDIUM;
use crate::radius as r;
use crate::recipe_parts::{anchored, body_text, modal, ring, scrim, shadowed, text, tween};
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use kivact::geometry::Edge;
use kivact::style::{
    Align, FlexDirection, Justify, Position, Style, Theme, ThemeMode, auto, pct, style,
};
use kivact::ui::DialogKind;
use kivact::ui::theme::{DialogState, DialogTheme, Look};

const DIALOG_WIDTH: f64 = 400.0;
const DIALOG_MAX_HEIGHT_PERCENT: f64 = 75.0;

/// The surface of the dialog's kind: a popover, a modal or alert, or a drawer.
fn surface(theme: &Theme, look: &Look<DialogState>, _base: Style) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    let gutter = theme.get(&s::SPACE_LAYER_VIEWPORT_GUTTER_VAR);
    let base = body_text(style())
        .flex_direction(FlexDirection::Column)
        .border_radius(r::RADIUS_CONTAINER)
        .background(c::BG_SURFACE)
        .focus_visible(ring(c::FOCUS_RING));
    match look.kind {
        DialogKind::Popover => anchored(
            shadowed(base.background(c::BG_POPOVER), shadows::low(dark))
                .padding(s::SPACE_SECTION_PADDING)
                .max_width(pct(100) - gutter * 2.0)
                .max_height(pct(100) - gutter * 2.0),
            0.0,
            -theme.get(&s::SPACE_S2_VAR),
        ),
        DialogKind::Drawer(side) => drawer(shadowed(base, shadows::high(dark)), side),
        DialogKind::Modal | DialogKind::AlertModal => modal(
            shadowed(base, shadows::high(dark))
                .width(DIALOG_WIDTH)
                .max_width(pct(100) - gutter * 2.0)
                .max_height(pct(DIALOG_MAX_HEIGHT_PERCENT))
                .padding(s::SPACE_DIALOG_PADDING)
                .gap(s::SPACE_DIALOG_PADDING),
            theme,
        ),
    }
}

/// A drawer: a full-height panel on `side` that slides out through the exit lifecycle.
fn drawer(base: Style, side: Edge) -> Style {
    let width = m::BORDER_WIDTH;
    let color = c::BORDER;
    let base = base.border_radius(r::RADIUS_NONE);
    let (base, hidden) = match side {
        Edge::Right => (
            base.width(DIALOG_WIDTH)
                .max_width(pct(100))
                .border_left(width, color),
            style().translate_x(pct(100)),
        ),
        Edge::Left => (
            base.width(DIALOG_WIDTH)
                .max_width(pct(100))
                .border_right(width, color),
            style().translate_x(pct(-100)),
        ),
        Edge::Top => (
            base.max_height(pct(100)).border_bottom(width, color),
            style().translate_y(pct(-100)),
        ),
        Edge::Bottom => (
            base.max_height(pct(100)).border_top(width, color),
            style().translate_y(pct(100)),
        ),
    };
    tween(base, MEDIUM).enter(hidden.clone()).exit(hidden)
}

/// The scrim behind the dialog; a drawer's scrim fades with it.
fn dialog_scrim(theme: &Theme, look: &Look<DialogState>, _base: Style) -> Style {
    let base = scrim(theme);
    if matches!(look.kind, DialogKind::Drawer(_)) {
        let hidden = style().opacity(0.0);
        tween(base, MEDIUM).enter(hidden.clone()).exit(hidden)
    } else {
        base
    }
}

/// The header: a drawer's header is padded like its body.
fn header(_theme: &Theme, look: &Look<DialogState>, _base: Style) -> Style {
    let base = style().flex_direction(FlexDirection::Column);
    if matches!(look.kind, DialogKind::Drawer(_)) {
        base.padding(s::SPACE_DIALOG_PADDING)
    } else {
        base
    }
}

/// The title: the second heading size in the primary color.
fn title(_theme: &Theme, _look: &Look<DialogState>, _base: Style) -> Style {
    text(style(), t::HEADING2, c::TEXT_PRIMARY)
}

/// The description: the supporting text, kept at the body size in an alert.
fn description(_theme: &Theme, look: &Look<DialogState>, _base: Style) -> Style {
    let base = style().color(c::TEXT_SECONDARY);
    if look.kind == DialogKind::AlertModal {
        base
    } else {
        base.font_size(t::SIZE_SM)
    }
}

/// The body: a column that fills the dialog; a drawer's body is padded.
fn body(_theme: &Theme, look: &Look<DialogState>, _base: Style) -> Style {
    let base = style().flex_direction(FlexDirection::Column).min_height(0);
    if matches!(look.kind, DialogKind::Drawer(_)) {
        base.padding_x(s::SPACE_DIALOG_PADDING)
    } else {
        base
    }
}

/// The footer: its actions at the end of a row; a drawer pins it to the bottom.
fn footer(_theme: &Theme, look: &Look<DialogState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Row)
        .justify_content(Justify::End)
        .gap(s::SPACE_S2);
    if matches!(look.kind, DialogKind::Drawer(_)) {
        base.margin_top(auto()).padding(s::SPACE_DIALOG_PADDING)
    } else {
        base
    }
}

/// The header's close button: a square at the top corner, holding the theme's `x` icon.
fn close(_theme: &Theme, _look: &Look<DialogState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .top(s::SPACE_S2)
        .right(s::SPACE_S2)
        .size(m::CONTROL_MD, m::CONTROL_MD)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_radius(r::RADIUS_ELEMENT)
        .color(c::TEXT_PRIMARY)
        .hover(style().background(c::OVERLAY_HOVER))
        .active(style().background(c::OVERLAY_PRESSED))
        .focus_visible(ring(c::FOCUS_RING))
}

/// Install the kit's recipes of the dialog.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        DialogTheme::default()
            .surface(surface)
            .scrim(dialog_scrim)
            .header(header)
            .title(title)
            .description(description)
            .body(body)
            .footer(footer)
            .close(close),
    )
}
