//! Role: Share Neutral transitions, outlines and composited control fills.
use crate::Tokens;
use kivact::motion::Transition;
use kivact::style::Color;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::TransitionProp;
use kivact::style::style;

pub(super) fn fast(t: &Tokens, base: Style) -> Style {
    base.transition(
        TransitionProp::All,
        Transition::tween(t.motion.fast as f64, t.motion.ease_standard.clone()),
    )
}

pub(super) fn ring(t: &Tokens, color: Color) -> Style {
    style().ring_with_offset(t.sizes.focus_ring_width, t.sizes.focus_ring_offset, color)
}

// CSS overlays are alpha-composited over the fill. Preserve transparency on
// secondary, ghost and danger so they also work over non-default surfaces.
pub(super) fn overlay(theme: &Theme, base: Color, top: Color) -> Color {
    let base = base.resolve(theme);
    let top = top.resolve(theme);
    let alpha = top.a + base.a * (1.0 - top.a);
    if alpha == 0.0 {
        return Color::TRANSPARENT;
    }
    let amount = top.a / alpha;
    let mix = |under, over| under * (1.0 - amount) + over * amount;
    Color::from_rgba_f64(
        mix(base.r, top.r),
        mix(base.g, top.g),
        mix(base.b, top.b),
        alpha,
    )
}
