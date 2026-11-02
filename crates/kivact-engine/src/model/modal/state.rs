//! Role: The modal group's state: admitted pointer drags and adopted gestures.
//!
//! The drags admitted before a modal opened may finish outside it; the
//! gestures the client adopted decline their pointer's click.

use std::cell::RefCell;
use std::collections::BTreeMap;

use kivact_wire::{NodeId, PointerId, RegionId};

/// A pointer's primary gesture that a canvas region took over: its click
/// activates nothing at that region.
#[derive(Clone, Copy)]
struct Adoption {
    node: NodeId,
    region: RegionId,
}

/// The node whose `Down` admitted each pointer's drag in progress, and the
/// gesture each pointer's client adopted, by pointer id (`PointerId::0`).
/// Both follow the order the events are dispatched in (admission is asked
/// right before each one), not the order the engine produced them. A drag
/// ends at the `Up` that leaves no button held, or at a `Cancel`. Interior
/// mutability: admission is asked of a shared model (`Reader::admit_input`),
/// and every call borrows the table for its own duration only, never across
/// a callback.
#[derive(Default)]
pub(crate) struct Admission {
    drags: RefCell<BTreeMap<u64, NodeId>>,
    adoptions: RefCell<BTreeMap<u64, Adoption>>,
}

impl Admission {
    /// End the admissions of nodes `live` does not accept (removed or
    /// leaving): the engine sends no `Cancel` to a node that went away
    /// while it held a pointer.
    pub(crate) fn forget_missing(&self, live: impl Fn(NodeId) -> bool) {
        self.drags.borrow_mut().retain(|_, node| live(*node));
        self.adoptions
            .borrow_mut()
            .retain(|_, adoption| live(adoption.node));
    }

    /// Record that `pointer`'s current gesture was adopted at `region` of
    /// `node`, replacing the pointer's earlier adoption.
    pub(crate) fn adopt(&self, pointer: PointerId, node: NodeId, region: RegionId) {
        self.adoptions
            .borrow_mut()
            .insert(pointer.0, Adoption { node, region });
    }

    /// End the adoption of `pointer`, if any; whether it covers the
    /// activation of `region` of `node`. An activation of the node's own
    /// action (no region) is never covered.
    pub(crate) fn take_adoption(
        &self,
        pointer: PointerId,
        node: NodeId,
        region: Option<RegionId>,
    ) -> bool {
        self.adoptions
            .borrow_mut()
            .remove(&pointer.0)
            .is_some_and(|adoption| adoption.node == node && region == Some(adoption.region))
    }

    /// End the adoption of `pointer`, if any.
    pub(crate) fn forget_adoption(&self, pointer: PointerId) {
        self.adoptions.borrow_mut().remove(&pointer.0);
    }

    /// The node that got the `Down` of `pointer`'s drag in progress.
    pub(crate) fn drag(&self, pointer: PointerId) -> Option<NodeId> {
        self.drags.borrow().get(&pointer.0).copied()
    }

    /// Admit a press of `pointer` at `node`: its drag is at `node` from now on.
    pub(crate) fn press(&self, pointer: PointerId, node: NodeId) {
        self.drags.borrow_mut().insert(pointer.0, node);
    }

    /// The gesture of `pointer` at `node` ended (its last button released, or
    /// cancelled).
    pub(crate) fn end(&self, pointer: PointerId, node: NodeId) {
        let mut drags = self.drags.borrow_mut();
        if drags.get(&pointer.0) == Some(&node) {
            drags.remove(&pointer.0);
        }
    }
}
