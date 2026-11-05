//! Role: Resolve measured native carousel page navigation without retaining geometry.
use kivact_core::geometry::Point;
use kivact_core::view::{NodeRef, ScrollBehavior};
use kivact_interaction::Snap;
pub(super) fn page(
    viewport: NodeRef,
    track: NodeRef,
    slides: &[NodeRef],
    vertical: bool,
    rtl: bool,
    next: bool,
    looping: bool,
    snap: Snap,
    padding: f64,
) -> Option<usize> {
    let view = viewport.rect().get_untracked()?;
    let content = track.rect().get_untracked()?;
    let current = viewport.scroll_offset().get_untracked();
    let offset = if vertical { current.y } else { current.x };
    let extent = if vertical { view.height } else { view.width };
    let limit = (if vertical {
        content.height
    } else {
        content.width
    } - extent)
        .max(0.0);
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
    track: NodeRef,
    vertical: bool,
    rtl: bool,
    next: bool,
    looping: bool,
) -> bool {
    let (Some(view), Some(content)) = (viewport.rect().get(), track.rect().get()) else {
        return false;
    };
    let offset = viewport.scroll_offset().get();
    let offset = if vertical { offset.y } else { offset.x };
    let limit = (if vertical {
        content.height - view.height
    } else {
        content.width - view.width
    })
    .max(0.0);
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
