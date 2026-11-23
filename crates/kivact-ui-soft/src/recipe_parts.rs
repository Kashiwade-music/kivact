//! Role: Share Astryx Neutral transitions, focus rings, field frames and composited
//! control fills between recipes.
use crate::color as c;
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, FAST, FAST_MAX, MEDIUM_MAX};
use crate::shadows;
use crate::space as s;
use crate::type_scale::{self as t, TypeStyle};
use kivact::custom::PartState;
use kivact::motion::Transition;
use kivact::style::{Color, Position, Shadow, Style, Theme, ThemeMode, TransitionProp, style};

/// The scale a floating layer starts from when it enters.
const LAYER_ENTER_SCALE: f64 = 0.95;

/// The standard short transition of a control's state change.
pub(crate) fn fast(base: Style) -> Style {
    base.transition(
        TransitionProp::All,
        Transition::tween(FAST as f64, EASE_STANDARD),
    )
}

/// The focus ring of a control: the kit's ring width and offset in `color`.
pub(crate) fn ring(color: Color) -> Style {
    style()
        .outline(m::FOCUS_RING_WIDTH, color)
        .outline_offset(m::FOCUS_RING_OFFSET)
}

// CSS overlays are alpha-composited over the fill. Preserve transparency on
// secondary, ghost and danger so they also work over non-default surfaces.
pub(crate) fn overlay(theme: &Theme, base: Color, top: Color) -> Color {
    let [tr, tg, tb, ta] = top.components(theme);
    let [br, bg, bb, ba] = base.components(theme);
    let alpha = ta + ba * (1.0 - ta);
    if alpha == 0.0 {
        return Color::TRANSPARENT;
    }
    let amount = ta / alpha;
    let mix = |under: f64, over: f64| under * (1.0 - amount) + over * amount;
    Color::from_rgba_f64(mix(br, tr), mix(bg, tg), mix(bb, tb), alpha)
}

/// The text of a type role in `color`, on `base`.
pub(crate) fn text(base: Style, role: TypeStyle, color: Color) -> Style {
    base.font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .font_weight(role.weight)
        .color(color)
}

/// The frame of a text field: its surface, border, hover and focus.
pub(crate) fn frame(theme: &Theme, input: &PartState, invalid: bool) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    let hover = if invalid {
        shadows::inset_error()
    } else {
        shadows::input_hover_ring(dark)
    };
    let focus = if invalid {
        style()
    } else {
        style()
            .border_color(c::ACCENT)
            .shadow(shadows::input_focus_ring(dark))
    };
    fast(
        style()
            .min_width(0)
            .background(c::BG_SURFACE)
            .border_radius(crate::radius::RADIUS_ELEMENT)
            .border(
                m::BORDER_WIDTH,
                if invalid {
                    c::TEXT_ERROR
                } else {
                    c::BORDER_EMPHASIZED
                },
            )
            .hover(if input.is_focused() {
                style()
            } else {
                style().shadow(hover)
            })
            .focus(focus),
    )
}

/// The text of an editor: body text with its placeholder and caret colors.
pub(crate) fn editor(base: Style) -> Style {
    text(base, crate::type_scale::BODY, c::TEXT_PRIMARY)
        .placeholder_color(c::TEXT_SECONDARY)
        .caret_color(c::TEXT_PRIMARY)
}

/// An editor without its own frame, for a composite input that draws the frame.
pub(crate) fn flat(theme: &Theme) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    let clear = Shadow {
        color: Color::TRANSPARENT,
        ..shadows::input_focus_ring(dark)
    };
    let clear_edge = style().border_color(Color::TRANSPARENT).shadow(clear);
    editor(style())
        .background(Color::TRANSPARENT)
        .border(0, Color::TRANSPARENT)
        .padding(0)
        .hover(clear_edge.clone())
        .focus(clear_edge)
}

/// The body text of a layer in the primary color, on `base`.
pub(crate) fn body_text(base: Style) -> Style {
    text(base, t::BODY, c::TEXT_PRIMARY)
}

/// The supporting text of a layer in the secondary color, on `base`.
pub(crate) fn supporting_text(base: Style) -> Style {
    text(base, t::SUPPORTING, c::TEXT_SECONDARY)
}

/// `base` with every property transitioning over `ms` with the standard easing.
pub(crate) fn tween(base: Style, ms: u64) -> Style {
    base.transition(
        TransitionProp::All,
        Transition::tween(ms as f64, EASE_STANDARD),
    )
}

/// `base` with the drop shadow `layers`, drawn in order.
pub(crate) fn shadowed(base: Style, layers: Vec<Shadow>) -> Style {
    layers.into_iter().fold(base, Style::shadow)
}

/// The layer that covers the window behind a modal: a blurred scrim.
pub(crate) fn scrim(theme: &Theme) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(c::OVERLAY_SCRIM)
        .backdrop_blur(theme.get(&s::SPACE_S0_5_VAR))
}

/// An anchored layer: it fades and scales in from its anchor, shifted by (`x`, `y`).
pub(crate) fn anchored(base: Style, x: f64, y: f64) -> Style {
    tween(base, FAST_MAX).enter(
        style()
            .opacity(0.0)
            .translate(x, y)
            .scale_xy(LAYER_ENTER_SCALE, LAYER_ENTER_SCALE),
    )
}

/// A modal layer: it fades in and rises a step from the page.
pub(crate) fn modal(base: Style, theme: &Theme) -> Style {
    tween(base, MEDIUM_MAX).enter(
        style()
            .opacity(0.0)
            .translate_y(theme.get(&s::SPACE_S4_VAR))
            .scale_xy(LAYER_ENTER_SCALE, LAYER_ENTER_SCALE),
    )
}
