//! Role: Layout state kept between passes: topology epoch, changes and relayout requests.

use crate::style::node::TStyle;
use std::cell::Cell;

/// A generation of the paired tree's topology: a cursor counted at another one is stale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Epoch(u64);

impl Epoch {
    /// The generation after this one.
    pub(crate) fn next(self) -> Epoch {
        Epoch(
            self.0
                .checked_add(1)
                .expect("invariant: layout epoch fits u64"),
        )
    }
}

// xtask-allow(state-fields): Topology generation and fixed traversal cursor are independent derived cache metadata (S-9); changed is a separate invalidation fact.
/// Pending work and traversal generations for one columns/tree allocation domain.
/// Keep it paired with the same NodeLayoutColumns and authoritative slot mapping.
/// Cloning all three produces independent derived state; mixing a state from an
/// unrelated allocation is unsupported. It owns no authoritative links or nodes.
#[derive(Clone, Debug)]
pub(crate) struct LayoutState {
    pub(crate) no_grid: TStyle,
    pub(crate) changed: bool,
    /// The shared generation of derived child traversal cursors.
    pub(crate) epoch: Epoch,
    /// The last indexed fixed child; this is a traversal cursor only.
    pub(crate) fixed_cursor: Cell<Option<(usize, kivact_types::NodeId)>>,
    /// Parents of absolute boxes whose layout was invalidated: each is laid
    /// out again on its own before the roots (`tree`).
    pub(crate) relayout: Vec<kivact_types::NodeId>,
}

impl Default for LayoutState {
    /// A new state has its first computation pending.
    fn default() -> Self {
        Self {
            no_grid: TStyle::default(),
            changed: true,
            epoch: Epoch::default(),
            fixed_cursor: Cell::default(),
            relayout: Vec::new(),
        }
    }
}

impl LayoutState {
    /// Mark authoritative topology or fixed membership changed and expire cursors.
    /// Call after every parent/child link, root, overlay or fixed-membership edit
    /// within this state's paired tree/columns allocation domain.
    /// Generation exhaustion is an invariant failure rather than a wrapped epoch.
    pub(crate) fn topology_changed(&mut self) {
        self.changed = true;
        self.epoch = self.epoch.next();
    }

    /// Consume the pending computation flag once before deciding whether to compute.
    /// Geometry reports and scrolling can still require work when this is false.
    pub(crate) fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }
}
