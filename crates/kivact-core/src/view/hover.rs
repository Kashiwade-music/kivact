//! Role: Hover state and pointer tracking.
//!
//! The pointer's hover as a reactive source of the view state: the node the
//! host last reported in `HoverChanged` (the hit itself, whatever pointer
//! listener or capture there is), which `NodeRef::hovered` reads.
//!
//! Invariants:
//! - The host owns hit testing and the hover chain; this module only holds
//!   its last report. The client records it inside the turn.
//! - The window's record holds the hover and the mouse position as signals
//!   made with the window (`open_window_record`). They exist before any view
//!   mounts, hold every later report for the window's lifetime and are
//!   disposed with the window.
//! - Once it exists, it is written only when a report arrives.
//!   `hovered` walks the core tree's parent chain of that node, which a
//!   later turn does not change for a live node: the core never reparents a
//!   mounted node (a keyed move stays under the same parent). Node ids are
//!   never reused, so a stale id never matches a node mounted later.

use std::panic::Location;

use crate::geometry::Point;
use kivact_runtime::Runtime;
use kivact_runtime::{Owner, Signal, Value};
use kivact_types::{FocusTarget, WindowId};

use crate::view::state::{ViewState, current_window};

/// The hover source of `window`, kept for the window's lifetime.
pub(crate) fn source(rt: &Runtime, window: WindowId) -> Signal<Option<FocusTarget>> {
    ViewState::of(rt)
        .with_window(window, |record| record.hover)
        .expect("invariant: the window is open")
}

/// The host reported that `node` is under the pointer now; a node already
/// unmounted on the core side hovers nothing.
pub(crate) fn set_hovered(rt: &Runtime, window: WindowId, node: Option<FocusTarget>) {
    let node = node.filter(|id| ViewState::tree(rt).get(id.node).is_some());
    // Only the hover source is written here: no owner applies.
    if let Some(hover) = ViewState::of(rt).with_window(window, |record| record.hover) {
        rt.enter(|| hover.set(node));
    }
}

/// The last mouse position of the calling owner's window in logical window
/// coordinates, independently of hit testing, capture or node handlers. It
/// is absent outside the window. Call during setup to observe movement
/// across gaps between floating views.
#[track_caller]
pub fn pointer_position() -> Value<Option<Point>> {
    let at = Location::caller();
    let rt = Runtime::current().expect("pointer_position requires an active App");
    let window = current_window(Owner::ambient("pointer_position", at));
    ViewState::of(&rt)
        .with_window(window, |record| record.mouse.into())
        .expect("invariant: the calling owner's window is open")
}

pub(crate) fn set_pointer_position(
    rt: &Runtime,
    window: WindowId,
    position: Option<kivact_types::Point>,
) {
    let position = position.map(|point| Point::new(point.x, point.y));
    if let Some(mouse) = ViewState::of(rt).with_window(window, |record| record.mouse) {
        rt.enter(|| mouse.set(position));
    }
}
