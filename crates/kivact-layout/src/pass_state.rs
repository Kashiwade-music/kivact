//! Role: Layout state kept between passes: topology epoch, changes and relayout requests.

use crate::style::node::TStyle;
use std::cell::Cell;

/// Pending work and traversal generations for one columns/tree allocation domain.
/// Keep it paired with the same NodeLayoutColumns and authoritative slot mapping.
/// Cloning all three produces independent derived state; mixing a state from an
/// unrelated allocation is unsupported. It owns no authoritative links or nodes.
#[derive(Clone, Debug, Default)]
pub struct LayoutState {
    pub(crate) no_grid: TStyle,
    pub(crate) changed: bool,
    /// The shared generation of derived child traversal cursors.
    pub(crate) epoch: u64,
    /// The last indexed fixed child; this is a traversal cursor only.
    pub(crate) fixed_cursor: Cell<Option<(usize, kivact_wire::NodeId)>>,
    /// Parents of absolute boxes whose layout was invalidated: each is laid
    /// out again on its own before the roots (`tree`).
    pub(crate) relayout: Vec<kivact_wire::NodeId>,
}

impl LayoutState {
    /// Mark authoritative topology or fixed membership changed and expire cursors.
    /// Call after initial tree population, before the first computation, and
    /// after every parent/child link, root, overlay or fixed-membership edit
    /// within this state's paired tree/columns allocation domain.
    /// Generation exhaustion is an invariant failure rather than a wrapped epoch.
    pub fn topology_changed(&mut self) {
        self.changed = true;
        self.epoch = self
            .epoch
            .checked_add(1)
            .expect("invariant: layout epoch fits u64");
    }

    /// Consume the pending computation flag once before deciding whether to compute.
    /// Geometry reports and scrolling can still require work when this is false.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }
}
