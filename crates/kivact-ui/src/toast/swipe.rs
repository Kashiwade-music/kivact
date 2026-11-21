//! Role: The look of a toast card that a swipe moves or dismissed.
use kivact_core::geometry::Edge;
use kivact_core::motion::{Easing, Transition};
use kivact_core::style::{Style, TransitionProp, pct, style, use_theme};
use kivact_interaction::SwipePhase;

fn sign(edge: Edge) -> f64 {
    match edge {
        Edge::Top | Edge::Left => -1.0,
        Edge::Bottom | Edge::Right => 1.0,
    }
}
fn shift(edge: Edge, distance: f64, base: Style) -> Style {
    match edge {
        Edge::Top | Edge::Bottom => base.translate_y(distance * sign(edge)),
        Edge::Left | Edge::Right => base.translate_x(distance * sign(edge)),
    }
}
/// The card's style: it follows the drag and leaves from where a
/// dismissing release let go. Only a dismissing swipe slides the card
/// off; any other dismissal keeps the card's own exit.
pub(super) fn card_style(edge: Edge, phase: SwipePhase, transition: &Transition) -> Style {
    let timing = if matches!(phase, SwipePhase::Dragging { .. }) {
        Transition::tween(0.0, Easing::Linear)
    } else {
        transition.clone()
    };
    let base = style().transition(TransitionProp::All, timing);
    let (travel, size, base) = match phase {
        SwipePhase::Dragging { offset, size } => (offset, size, base),
        SwipePhase::Dismissed { travel, size } => {
            let away = match edge {
                Edge::Top | Edge::Bottom => {
                    style().opacity(0.0).translate_y(pct(120.0 * sign(edge)))
                }
                Edge::Left | Edge::Right => {
                    style().opacity(0.0).translate_x(pct(120.0 * sign(edge)))
                }
            };
            (travel, size, base.exit(away.scale(1.0)))
        }
        SwipePhase::Rest | SwipePhase::Pressed => return base,
    };
    let progress = (travel / size).clamp(0.0, 1.0);
    shift(edge, travel, base)
        .opacity(1.0 - 0.4 * progress)
        .scale(1.0 - 0.02 * progress)
}
/// The style of the card's row. After a qualifying swipe the row closes
/// the gap only once the card has slid off: its size transition waits
/// for the swipe's exit (`exit`), so the row's collapse does not carry
/// the card past the region's edge before it is seen leaving. A spring
/// exit has no set length, and the row then collapses at once.
pub(super) fn row_style(phase: SwipePhase, exit: &Transition) -> Style {
    if !matches!(phase, SwipePhase::Dismissed { .. }) {
        return style();
    }
    let theme = use_theme().get();
    let row = theme
        .ext::<crate::toast::look::ToastTheme>()
        .row_transition_value()
        .clone();
    match (row, exit.duration_ms()) {
        (Some(row), Some(wait)) => {
            let delay = row.delay_ms + wait;
            style().transition(TransitionProp::Size, row.delay(delay))
        }
        _ => style(),
    }
}
