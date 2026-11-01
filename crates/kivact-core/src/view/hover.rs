//! Role: Hover state and pointer tracking.
//!
//! The pointer's hover as a reactive source of the view state: the node the
//! host last reported in `HoverChanged` (the hit itself, whatever pointer
//! listener or capture there is), which `NodeRef::hovered` reads.
//!
//! Invariants:
//! - The host owns hit testing and the hover chain; this module only holds
//!   its last report. The client records it inside the turn.
//! - The last reported node is always kept as a plain value
//!   (`ViewState::hovered`). The graph source is created by the first
//!   `NodeRef::hovered`, under the root owner, and lives as long as the
//!   App; an App that never calls it has no hover source and a report writes
//!   nothing to the graph. The source starts at the node reported so far.
//! - Once it exists, it is written only when a report arrives.
//!   `hovered` walks the core tree's parent chain of that node, which a
//!   later turn does not change for a live node: the core never reparents a
//!   mounted node (a keyed move stays under the same parent). Node ids are
//!   never reused, so a stale id never matches a node mounted later.

use std::panic::Location;

use kivact_reactive::Runtime;
use kivact_reactive::Signal;
use kivact_wire::NodeId;

use crate::view::state::ViewState;

/// The hover source of `rt`'s App, created under the root owner by the first
/// call (at the node the host reported last) and kept for the App's lifetime.
pub(crate) fn source(rt: &Runtime, at: &'static Location<'static>) -> Signal<Option<NodeId>> {
    let state = ViewState::of(rt);
    if let Some(source) = state.hover() {
        return source;
    }
    let owner = {
        let tree = ViewState::tree(rt);
        tree.root
            .and_then(|root| tree.get(root))
            .expect("invariant: the App root exists before a hover source is read")
            .owner
    };
    let source = owner.signal(state.hovered(), at);
    state.set_hover(source);
    source
}

/// The host reported that `node` is under the pointer now; a node already
/// unmounted on the core side hovers nothing.
pub(crate) fn set_hovered(rt: &Runtime, node: Option<NodeId>) {
    let node = node.filter(|id| ViewState::tree(rt).get(*id).is_some());
    let state = ViewState::of(rt);
    state.set_hovered(node);
    let Some(source) = state.hover() else {
        return;
    };
    // Only the hover source is written here: no owner applies.
    rt.enter(|| {
        source.set(node);
    })
}
