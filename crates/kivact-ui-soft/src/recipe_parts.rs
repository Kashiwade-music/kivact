//! Role: Share Astryx Neutral transitions, focus rings, field frames and composited control fills between recipes.
use crate::color as c;
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, FAST};
use crate::shadows;
use crate::type_scale::TypeStyle;
use kivact::custom::PartState;
use kivact::motion::Transition;
use kivact::style::{Color, Shadow, Style, Theme, ThemeMode, TransitionProp, style};

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
