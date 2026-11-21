//! Role: Install Astryx Neutral's recipes of the overlays group: the pickers, the menus, the dialogs and the popups.
use crate::color as c;
use crate::radius as r;
use crate::recipe_parts::{anchored, body_text, shadowed};
use crate::shadows;
use crate::space as s;
use kivact::interaction::PopupState;
use kivact::style::{FlexDirection, Style, Theme, ThemeMode, pct, style};
use kivact::ui::theme::{HoverCardSurfaceState, HoverCardTheme, Look, TooltipTheme};
use kivact::view::Placement;

const TOOLTIP_CONTENT_MAX_WIDTH: f64 = 300.0;

/// The tooltip: a small inverted surface that rises from its trigger.
fn tooltip(theme: &Theme, _look: &Look<PopupState>, _base: Style) -> Style {
    let gap = theme.get(&s::SPACE_S2_VAR);
    let base = body_text(style())
        .color(c::BG_SURFACE)
        .background(c::TEXT_PRIMARY)
        .border_radius(r::RADIUS_CONTAINER)
        .padding_x(s::SPACE_S2)
        .padding_y(s::SPACE_S1)
        .max_width(TOOLTIP_CONTENT_MAX_WIDTH + gap * 2.0);
    anchored(base, 0.0, gap)
}

/// The hover card: a neutral preview surface that enters from its placement's side.
fn hover_card(theme: &Theme, look: &Look<HoverCardSurfaceState>, _base: Style) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    let base = shadowed(body_text(style()), shadows::med(dark))
        .flex_direction(FlexDirection::Column)
        .background(c::BG_SURFACE)
        .border_radius(r::RADIUS_CONTAINER)
        .max_width(pct(100) - 32.0)
        .padding(s::SPACE_S3);
    let gap = theme.get(&s::SPACE_S2_VAR);
    let (x, y) = match look.placement {
        Placement::Above => (0.0, gap),
        Placement::Below => (0.0, -gap),
        Placement::Start => (gap, 0.0),
        Placement::End => (-gap, 0.0),
        _ => (0.0, 0.0),
    };
    anchored(base, x, y)
}

/// Install the kit's recipes of the overlays group.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(HoverCardTheme::default().surface(hover_card))
        .with(TooltipTheme::default().surface(tooltip));
    let theme = crate::recipes_overlays_select::install(theme);
    let theme = crate::recipes_overlays_menu::install(theme);
    let theme = crate::recipes_overlays_dialog::install(theme);
    crate::recipes_overlays_command::install(theme)
}
