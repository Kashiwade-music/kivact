//! Role: Style the standard tooltip and notification region.
use crate::Tokens;
use crate::recipes_dialog::{anchored, body_text, focus, motion, shadows};
use kivact::interaction::{PopupState, ToastState};
use kivact::primitive::PartState;
use kivact::style::{
    Align, Color, Direction, Justify, Position, Style, Theme, ThemeMode, pct, style,
};
use kivact::ui::{Look, ToastGroupState, ToastRecipes, Tone, TooltipRecipes};

const TOOLTIP_CONTENT_MAX_WIDTH: f64 = 300.0;
const TOAST_WIDTH: f64 = 400.0;

pub(crate) fn tooltip(r: TooltipRecipes) -> TooltipRecipes {
    r.surface(|_| {
        |t: &Theme, _: &Look<PopupState>| {
            let p = t.get::<Tokens>();
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
            let p = t.get::<Tokens>();
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
    let p = t.get::<Tokens>();
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
    let p = t.get::<Tokens>();
    body_text(&p).color(foreground(t, s, &p))
}

fn close(t: &Theme, s: &Look<ToastState>) -> Style {
    let p = t.get::<Tokens>();
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
