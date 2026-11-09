//! Role: Resolve measured native carousel page navigation without retaining geometry.
use kivact_core::geometry::Point;
use kivact_core::view::{NodeRef, ScrollBehavior};
use kivact_interaction::Snap;
pub(super) struct PageRequest {
    // xtask-allow(state-fields): passive record - vertical
    pub(super) vertical: bool,
    // xtask-allow(state-fields): passive record - rtl
    pub(super) rtl: bool,
    // xtask-allow(state-fields): passive record - next
    pub(super) next: bool,
    // xtask-allow(state-fields): passive record - looping
    pub(super) looping: bool,
    pub(super) snap: Snap,
    pub(super) padding: f64,
}
pub(super) fn page(viewport: NodeRef, slides: &[NodeRef], request: PageRequest) -> Option<usize> {
    let PageRequest {
        vertical,
        rtl,
        next,
        looping,
        snap,
        padding,
    } = request;
    let view = viewport.rect().get_untracked()?;
    let current = viewport.scroll_offset().get_untracked();
    let offset = if vertical { current.y } else { current.x };
    let extent = if vertical { view.height } else { view.width };
    let (_, limit) = bounds(
        viewport,
        (slides.first().copied(), slides.last().copied()),
        vertical,
    )?;
    let forward = vertical || !rtl;
    let forward = if next { forward } else { !forward };
    let first = slides.first()?.rect().get_untracked()?;
    let first = if vertical { first.height } else { first.width };
    let distance = (extent - first * 0.5).max(first);
    let wrapped = looping
        && if forward {
            offset >= limit - 1.0
        } else {
            offset <= 1.0
        };
    let target = if wrapped {
        if forward { 0.0 } else { limit }
    } else {
        (offset + if forward { distance } else { -distance }).clamp(0.0, limit)
    };
    if snap == Snap::None {
        viewport.scroll_to(
            if vertical {
                Point {
                    x: current.x,
                    y: target,
                }
            } else {
                Point {
                    x: target,
                    y: current.y,
                }
            },
            if kivact_core::platform::is_motion_reduced().get_untracked() {
                ScrollBehavior::Instant
            } else {
                ScrollBehavior::Smooth(kivact_core::motion::Easing::EaseOut)
            },
        );
        return None;
    }
    slides
        .iter()
        .enumerate()
        .filter_map(|(index, reference)| {
            let rect = reference.rect().get_untracked()?;
            let delta = if vertical {
                rect.y - view.y - padding
            } else if rtl {
                rect.right() - view.right() + padding
            } else {
                rect.x - view.x - padding
            };
            let position = (offset + delta).clamp(0.0, limit);
            (wrapped
                || if forward {
                    position > offset + 1.0
                } else {
                    position < offset - 1.0
                })
            .then_some((index, position))
        })
        .min_by(|a, b| (a.1 - target).abs().total_cmp(&(b.1 - target).abs()))
        .map(|p| p.0)
}

pub(super) fn can_scroll(
    viewport: NodeRef,
    edges: (Option<NodeRef>, Option<NodeRef>),
    vertical: bool,
    rtl: bool,
    next: bool,
    looping: bool,
) -> bool {
    let Some((offset, limit)) = bounds(viewport, edges, vertical) else {
        return false;
    };
    let forward = if next {
        vertical || !rtl
    } else {
        !vertical && rtl
    };
    limit > 1.0
        && (looping
            || if forward {
                offset < limit - 1.0
            } else {
                offset > 1.0
            })
}

/// Derive content extent from slide geometry: a flex track's own box can be viewport-sized while its children overflow.
pub(super) fn bounds(
    viewport: NodeRef,
    edges: (Option<NodeRef>, Option<NodeRef>),
    vertical: bool,
) -> Option<(f64, f64)> {
    let view = viewport.rect().get()?;
    let first = edges.0?.rect().get()?;
    let last = edges.1?.rect().get()?;
    let point = viewport.scroll_offset().get();
    let (offset, extent, visible) = if vertical {
        (
            point.y,
            first.bottom().max(last.bottom()) - first.y.min(last.y),
            view.height,
        )
    } else {
        (
            point.x,
            first.right().max(last.right()) - first.x.min(last.x),
            view.width,
        )
    };
    Some((offset, (extent - visible).max(0.0)))
}
