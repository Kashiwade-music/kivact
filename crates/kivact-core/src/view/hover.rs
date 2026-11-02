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
//!   (`Hover::Plain`); that call turns it into a graph source under the root
//!   owner (`Hover::Reactive`), which holds every later report for the App's
//!   lifetime. An App that never calls it writes nothing to the graph. The
//!   source starts at the node reported so far.
//! - Once it exists, it is written only when a report arrives.
//!   `hovered` walks the core tree's parent chain of that node, which a
//!   later turn does not change for a live node: the core never reparents a
//!   mounted node (a keyed move stays under the same parent). Node ids are
//!   never reused, so a stale id never matches a node mounted later.

use std::panic::Location;

use kivact_reactive::Runtime;
use kivact_reactive::Signal;
use kivact_wire::NodeId;

use crate::view::state::{Hover, ViewState};

/// The hover source of `rt`'s App, created under the root owner by the first
/// call (at the node the host reported last) and kept for the App's lifetime.
pub(crate) fn source(rt: &Runtime, at: &'static Location<'static>) -> Signal<Option<NodeId>> {
    let state = ViewState::of(rt);
    let last = match state.hover() {
        Hover::Reactive(source) => return source,
        Hover::Plain(last) => last,
    };
    let owner = ViewState::tree(rt).root_owner();
    let source = owner.signal(last, at);
    state.set_hover(Hover::Reactive(source));
    source
}

/// The host reported that `node` is under the pointer now; a node already
/// unmounted on the core side hovers nothing.
pub(crate) fn set_hovered(rt: &Runtime, node: Option<NodeId>) {
    let node = node.filter(|id| ViewState::tree(rt).get(*id).is_some());
    let state = ViewState::of(rt);
    match state.hover() {
        Hover::Plain(_) => state.set_hover(Hover::Plain(node)),
        // Only the hover source is written here: no owner applies.
        Hover::Reactive(source) => rt.enter(|| source.set(node)),
    }
}
