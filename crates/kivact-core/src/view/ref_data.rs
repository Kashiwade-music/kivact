//! Role: Passive reference binding lifecycle and layout subscriptions.

use kivact_runtime::{Owner, Signal, Value};
use kivact_style::draw::Rect;
use kivact_types::NodeId;
use std::cell::Cell;

/// What a [`NodeRef`](super::NodeRef) is bound to.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Bound {
    /// A host node.
    Node(NodeId),
    /// A canvas region of a host node, with that region's pointer hover.
    Region {
        node: NodeId,
        region: kivact_style::draw::RegionId,
    },
}

impl Bound {
    /// The host node, whole or by region.
    pub(crate) fn node(self) -> NodeId {
        match self {
            Bound::Node(node) | Bound::Region { node, .. } => node,
        }
    }

    /// The focus target the binding names.
    pub(crate) fn target(self) -> kivact_types::FocusTarget {
        match self {
            Bound::Node(node) => kivact_types::FocusTarget { node, region: None },
            Bound::Region { node, region, .. } => kivact_types::FocusTarget {
                node,
                region: Some(region),
            },
        }
    }
}

/// What a [`NodeRef`](super::NodeRef) stores under its owner: the bound
/// target and its reactive readers.
pub(crate) struct NodeRefState {
    pub(crate) owner: Owner,
    /// The one source of binding and detachment. Tracked readers `get` it;
    /// untracked paths use `try_get_untracked`, where a disposed signal reads as unbound.
    pub(crate) bound: Signal<RefState>,
    /// What the readers called so far need on every node the ref binds to.
    pub(crate) observation: Cell<RefObservation>,
    // xtask-allow(state-fields): cache: the rect memo of the binding, created under the ref's owner by the first NodeRef::rect call; absent until then
    pub(crate) rect_value: Cell<Option<Value<Option<Rect>>>>,
    // xtask-allow(state-fields): cache: the scroll offset memo of the binding, created under the ref's owner by the first NodeRef::scroll_offset call; absent until then
    pub(crate) scroll_value: Cell<Option<Value<crate::geometry::Point>>>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum RefState {
    Unbound,
    Bound(Bound),
    Detached,
}
impl RefState {
    pub(crate) fn bound(self) -> Option<Bound> {
        match self {
            Self::Bound(bound) => Some(bound),
            Self::Unbound | Self::Detached => None,
        }
    }
}
/// What a reference's readers need on every node it binds to, from the first
/// such reader on (never lowered).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RefObservation {
    /// No reader of the node's geometry or status yet: a bound node gets no
    /// `NodeObservation` (a region binding always does).
    Unrequested,
    /// The node's graph sources (`NodeObservation`): scroll offset, image
    /// status, scroll progress.
    Sources,
    /// The sources and the engine's layout reports of the node (`NodeRef::rect`).
    SourcesAndLayout,
}
