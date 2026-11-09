//! Role: Hover state and pointer tracking.
//!
//! The pointer's hover as a reactive source of the view state: the node the
//! host last reported in `HoverChanged` (the hit itself, whatever pointer
//! listener or capture there is), which `NodeRef::hovered` reads.
//!
//! Invariants:
//! - The host owns hit testing and the hover chain; this module only holds
//!   its last report. The client records it inside the turn.
//! - Before the first `NodeRef::hovered` the last report is a plain value
//!   (`PointerObservation::Plain`); that call turns it into a graph source under the root
//!   owner (`PointerObservation::Reactive`), which holds every later report for the App's
//!   lifetime. An App that never calls it writes nothing to the graph. The
//!   source starts at the node reported so far.
//! - Once it exists, it is written only when a report arrives.
//!   `hovered` walks the core tree's parent chain of that node, which a
//!   later turn does not change for a live node: the core never reparents a
//!   mounted node (a keyed move stays under the same parent). Node ids are
//!   never reused, so a stale id never matches a node mounted later.

use std::panic::Location;

use crate::geometry::Point;
use kivact_runtime::Runtime;
use kivact_runtime::{Signal, Value};
use kivact_types::{FocusTarget, WindowId};

use crate::view::state::{PointerObservation, ViewState, current_window};

/// The hover source of `window`'s App, created under the window's root
/// owner by the first call (at the node the host reported last) and kept for
/// the window's lifetime.
pub(crate) fn source(
    rt: &Runtime,
    window: WindowId,
    at: &'static Location<'static>,
) -> Signal<Option<FocusTarget>> {
    let state = ViewState::of(rt);
    let last = match state.hover(window) {
        PointerObservation::Reactive(source) => return source,
        PointerObservation::Plain(last) => last,
    };
    let owner = ViewState::tree(rt).owner_of(
        ViewState::tree(rt)
            .root(window)
            .expect("invariant: the window is open"),
    );
    let source = owner.signal(last, at);
    state.set_hover(window, PointerObservation::Reactive(source));
    source
}

/// The host reported that `node` is under the pointer now; a node already
/// unmounted on the core side hovers nothing.
pub(crate) fn set_hovered(rt: &Runtime, window: WindowId, node: Option<FocusTarget>) {
    let node = node.filter(|id| ViewState::tree(rt).get(id.node).is_some());
    let state = ViewState::of(rt);
    match state.hover(window) {
        PointerObservation::Plain(_) => state.set_hover(window, PointerObservation::Plain(node)),
        // Only the hover source is written here: no owner applies.
        PointerObservation::Reactive(source) => rt.enter(|| source.set(node)),
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
    let window = current_window(kivact_runtime::Owner::ambient("pointer_position", at));
    let state = ViewState::of(&rt);
    let last = match state.mouse(window) {
        PointerObservation::Reactive(source) => return source.into(),
        PointerObservation::Plain(last) => last,
    };
    let owner = ViewState::tree(&rt).owner_of(
        ViewState::tree(&rt)
            .root(window)
            .expect("invariant: the calling owner's window is open"),
    );
    let source = owner.signal(last, at);
    state.set_mouse(window, PointerObservation::Reactive(source));
    source.into()
}

pub(crate) fn set_pointer_position(
    rt: &Runtime,
    window: WindowId,
    position: Option<kivact_types::Point>,
) {
    let position = position.map(|point| Point::new(point.x, point.y));
    let state = ViewState::of(rt);
    match state.mouse(window) {
        PointerObservation::Plain(_) => {
            state.set_mouse(window, PointerObservation::Plain(position))
        }
        PointerObservation::Reactive(source) => rt.enter(|| source.set(position)),
    }
}
