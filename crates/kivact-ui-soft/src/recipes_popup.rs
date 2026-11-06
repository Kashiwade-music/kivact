//! Role: Style the standard tooltip and notification region.
use crate::Tokens;
use crate::recipes_dialog::{anchored, body_text, focus, motion, shadows};
use kivact::custom::PartState;
use kivact::interaction::PopupState;
use kivact::interaction::ToastState;
use kivact::interaction::Tone;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::ThemeMode;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::Look;
use kivact::ui::ToastGroupState;
use kivact::ui::ToastRecipes;
use kivact::ui::TooltipRecipes;

const TOOLTIP_CONTENT_MAX_WIDTH: f64 = 300.0;
const TOAST_WIDTH: f64 = 400.0;

pub(crate) fn tooltip(r: TooltipRecipes) -> TooltipRecipes {
    r.surface(|_| {
        |t: &Theme, _: &Look<PopupState>| {
            let p = t.ext::<Tokens>();
            let base = body_text(&p)
                .color(p.colors.bg_surface)
                .background(p.colors.text_primary)
                .radius(p.radius.container)
                .padding_x(p.spacing.s2)
                .padding_y(p.spacing.s1)
                .max_width(TOOLTIP_CONTENT_MAX_WIDTH + p.spacing.s2 * 2.0);
            anchored(base, &p, 0.0, p.spacing.s2)
        }
    })
}

pub(crate) fn toast(r: ToastRecipes) -> ToastRecipes {
    r.region(|_| {
        |t: &Theme, _: &Look<PartState>| {
            let p = t.ext::<Tokens>();
            let gutter = p.spacing.layer_viewport_gutter;
            style()
                .position(Position::Fixed)
                .bottom(gutter)
                .right(gutter)
                .width(TOAST_WIDTH)
                .max_width(pct(100) - gutter * 2.0)
                .direction(Direction::Column)
        }
    })
    .group(|_| |_: &Theme, _: &Look<ToastGroupState>| style().direction(Direction::Column))
    .toast(|_| card)
    .title(|_| text)
    .body(|_| text)
    .close(|_| close)
    .icon(|_| |_: &Theme, _: &Look<ToastState>| style().size(0, 0))
}

fn card(t: &Theme, s: &Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    let fill = if s.tone == Tone::Danger {
        p.colors.bg_error_inverted
    } else {
        p.colors.bg_inverted
    };
    let base = shadows(body_text(&p), &p.shadows.med)
        .direction(Direction::Row)
        .align_items(Align::Start)
        .gap(p.spacing.s3)
        .padding(p.spacing.dialog_padding)
        .radius(p.radius.container)
        .background(fill)
        .color(foreground(t, s, &p));
    let hidden = style().opacity(0.0).translate_y(p.spacing.s2);
    motion(base, &p, p.motion.fast)
        .enter(hidden.clone())
        .exit(hidden)
}

fn foreground(t: &Theme, s: &Look<ToastState>, p: &Tokens) -> Color {
    if t.mode() == ThemeMode::Dark && s.tone != Tone::Danger {
        p.colors.on_light
    } else {
        p.colors.on_dark
    }
}

fn text(t: &Theme, s: &Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    body_text(&p).color(foreground(t, s, &p))
}

fn close(t: &Theme, s: &Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    let inverted = if t.mode() == ThemeMode::Dark && s.tone != Tone::Danger {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    };
    let media = Tokens::new(inverted);
    let overhang = (p.sizes.control_sm - p.typography.body.line_height_px) / 2.0;
    style()
        .size(p.sizes.control_sm, p.sizes.control_sm)
        .shrink(0)
        .margin_top(-overhang)
        .margin_right(-overhang)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .radius(p.radius.element)
        .color(foreground(t, s, &p))
        .hover(style().background(media.colors.overlay_hover))
        .pressed(style().background(media.colors.overlay_pressed))
        .focus_visible(focus(&media))
}

/// Neutral preview surface, including side-aware entrance motion.
pub(crate) fn hover_card(r: kivact::ui::HoverCardRecipes) -> kivact::ui::HoverCardRecipes {
    r.surface(|_| {
        |theme: &Theme, state: &Look<kivact::ui::HoverCardSurfaceState>| {
            let p = theme.ext::<Tokens>();
            let base = shadows(body_text(&p), &p.shadows.med)
                .direction(Direction::Column)
                .background(p.colors.bg_surface)
                .radius(p.radius.container)
                .max_width(pct(100) - 32.0)
                .padding(p.spacing.s3);
            let (x, y) = match state.placement {
                kivact::view::Placement::Above => (0.0, p.spacing.s2),
                kivact::view::Placement::Below => (0.0, -p.spacing.s2),
                kivact::view::Placement::Start => (p.spacing.s2, 0.0),
                kivact::view::Placement::End => (-p.spacing.s2, 0.0),
                _ => (0.0, 0.0),
            };
            anchored(base, &p, x, y)
        }
    })
}
