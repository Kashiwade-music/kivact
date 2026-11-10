//! Role: Scroll-driven progress: a scroller's position and a node's travel
//! through its scroller's viewport, as reactive values in `0..=1`.
//!
//! Both are memos over the host's geometry reports: the scroller's offset
//! and extents (`ScrollChanged`) and the window layout boxes of the nodes
//! (`LayoutChanged`). A node that is not bound reads as 0.

use kivact_runtime::{Value, memo, require_setup};
use kivact_types::Orientation;

use crate::geometry::Rect;
use crate::view::node_ref::NodeRef;
use crate::view::tree::ScrollState;

/// Which part of the scroller's viewport a node's progress covers (the CSS
/// `view()` ranges).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewRange {
    /// From the node entering the viewport from the end until it leaves at the start.
    Cover,
    /// From the node fully inside the viewport until it is fully outside.
    Contain,
    /// From the node entering the viewport from the end until it is fully inside.
    Entry,
    /// From the node starting to leave until it has left at the start.
    Exit,
}

/// 0 at the start of `scroller`'s scroll range along `axis`, 1 at its end (0
/// if it cannot scroll, or for `Orientation::Unspecified`).
#[track_caller]
pub fn scroll_progress(scroller: NodeRef, axis: Orientation) -> Value<f64> {
    require_setup("scroll_progress");
    scroller.require_observation(
        crate::view::state::RefObservation::Sources,
        std::panic::Location::caller(),
    );
    memo(move || {
        scroller
            .tracked_observation()
            .and_then(|observation| observation.scroll_geometry.try_get())
            .map_or(0.0, |state| scroll_ratio(state, axis))
    })
}

/// How far `target` has travelled through `scroller`'s viewport along `axis`,
/// for `range` (the CSS `view()` timeline). The window layout boxes are used,
/// so the target's own transform is ignored. 0 when either is unbound or the
/// axis is `Orientation::Unspecified`.
#[track_caller]
pub fn view_progress(
    target: NodeRef,
    scroller: NodeRef,
    axis: Orientation,
    range: ViewRange,
) -> Value<f64> {
    require_setup("view_progress");
    let _ = target.rect();
    let _ = scroller.rect();
    memo(move || {
        let (Some(target), Some(scroller)) = (layout_rect(target), layout_rect(scroller)) else {
            return 0.0;
        };
        let (Some((t_start, t_len)), Some((s_start, s_len))) =
            (span(target, axis), span(scroller, axis))
        else {
            return 0.0;
        };
        view_ratio(range, s_len, t_len, t_start - s_start)
    })
}

/// The window layout box of `node`, tracked.
fn layout_rect(node: NodeRef) -> Option<Rect> {
    node.tracked_observation()?
        .measured
        .try_get()
        .flatten()
        .map(|measured| measured.layout_rect)
}

/// The start and length of `rect` along `axis`.
fn span(rect: Rect, axis: Orientation) -> Option<(f64, f64)> {
    match axis {
        Orientation::Horizontal => Some((rect.x, rect.width)),
        Orientation::Vertical => Some((rect.y, rect.height)),
        Orientation::Unspecified => None,
    }
}

/// The scroll offset over the scroll range along `axis`, clamped to `0..=1`.
fn scroll_ratio(state: ScrollState, axis: Orientation) -> f64 {
    let (offset, viewport, content) = match axis {
        Orientation::Horizontal => (state.offset.x, state.viewport.width, state.content.width),
        Orientation::Vertical => (state.offset.y, state.viewport.height, state.content.height),
        Orientation::Unspecified => return 0.0,
    };
    ratio(offset, content - viewport)
}

/// The progress of a target of length `t` at `r` from the viewport's start,
/// in a viewport of length `v`, for `range`.
fn view_ratio(range: ViewRange, v: f64, t: f64, r: f64) -> f64 {
    let (num, den) = match range {
        ViewRange::Cover => (v - r, v + t),
        ViewRange::Contain if t <= v => (v - r - t, v - t),
        ViewRange::Contain => (-r, t - v),
        ViewRange::Entry => (v - r, t.min(v)),
        ViewRange::Exit => {
            let end = if t <= v { 0.0 } else { v - t };
            (end - r, end + t)
        }
    };
    ratio(num, den)
}

/// `num / den` clamped to `0..=1`; 0 when the denominator is not positive.
fn ratio(num: f64, den: f64) -> f64 {
    if den <= 0.0 {
        return 0.0;
    }
    let value = num / den;
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}
