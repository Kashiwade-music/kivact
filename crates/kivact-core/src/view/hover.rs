//! Role: Hover state and pointer tracking.
//!
//! The pointer's hover as a reactive source of the view state: the node the
//! host last reported in `HoverChanged` (the hit itself, whatever pointer
//! listener or capture there is), which `NodeRef::hovered` reads.
//!
//! Invariants:
//! - The host owns hit testing and the hover chain; this module only holds
//!   its last report. The client records it inside the turn.
//! - The window installs the last report as a graph source under its root
//!   owner (`PointerObservation::Reactive`), as it does the mouse source: it
//!   exists before any view mounts and holds every later report for the
//!   window's lifetime. A record still holding `PointerObservation::Plain`
//!   is turned into one on first use.
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

use crate::view::state::{PointerObservation, ViewState, current_window};

/// Install the hover source of `window` under its root `owner`: a window
/// resource, like the mouse source, that exists before any view mounts and is
/// disposed with the window. Keeps an existing source.
pub(crate) fn install(
    rt: &Runtime,
    window: WindowId,
    owner: Owner,
    at: &'static Location<'static>,
) {
    let state = ViewState::of(rt);
    if let PointerObservation::Plain(last) = state.hover(window) {
        state.set_hover(window, PointerObservation::Reactive(owner.signal(last, at)));
    }
}

/// The hover source of `window`'s App, installed with the window under its
/// root owner and kept for the window's lifetime. A plain record is turned
/// into the source on first use, starting at the node the host reported last.
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
