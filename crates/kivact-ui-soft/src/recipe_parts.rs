//! Role: Share Astryx Neutral transitions, focus rings and composited control fills between recipes.
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, FAST};
use kivact::motion::Transition;
use kivact::style::{Color, Style, Theme, TransitionProp, style};

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
