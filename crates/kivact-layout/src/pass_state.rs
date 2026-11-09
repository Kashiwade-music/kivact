//! Role: Layout state kept between passes: topology epoch, pending work and relayout requests.

use crate::style::node::TStyle;
use kivact_types::NodeId;
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

/// The pending computation: whether one has to run, and the absolute parents
/// to lay out again before the roots (`tree`).
#[derive(Clone, Debug)]
pub(crate) enum Pending {
    /// Nothing to do: the last computation covered every change.
    Clean,
    /// A computation must run; each queued parent is an absolute box's parent
    /// whose layout was invalidated.
    Dirty { relayout: Vec<NodeId> },
}

impl Pending {
    /// The queued parents; empty for `Clean`.
    pub(crate) fn into_relayout(self) -> Vec<NodeId> {
        match self {
            Pending::Clean => Vec::new(),
            Pending::Dirty { relayout } => relayout,
        }
    }
}

/// Pending work and traversal generations for one columns/tree allocation domain.
/// Keep it paired with the same NodeLayoutColumns and authoritative slot mapping.
/// Cloning all three produces independent derived state; mixing a state from an
/// unrelated allocation is unsupported. It owns no authoritative links or nodes.
#[derive(Clone, Debug)]
pub(crate) struct LayoutState {
    pub(crate) no_grid: TStyle,
    /// The pending work: marked by every invalidation, taken by the computation.
    pending: Pending,
    /// The shared generation of derived child traversal cursors.
    pub(crate) epoch: Epoch,
    // xtask-allow(state-fields): cache: a traversal cursor counted at one epoch
    /// The last indexed fixed child; this is a traversal cursor only.
    pub(crate) fixed_cursor: Cell<Option<(usize, NodeId)>>,
}

impl Default for LayoutState {
    /// A new state has its first computation pending.
    fn default() -> Self {
        Self {
            no_grid: TStyle::default(),
            pending: Pending::Dirty {
                relayout: Vec::new(),
            },
            epoch: Epoch::default(),
            fixed_cursor: Cell::default(),
        }
    }
}

impl LayoutState {
    /// Mark authoritative topology or fixed membership changed and expire cursors.
    /// Call after every parent/child link, root, overlay or fixed-membership edit
    /// within this state's paired tree/columns allocation domain.
    /// Generation exhaustion is an invariant failure rather than a wrapped epoch.
    pub(crate) fn topology_changed(&mut self) {
        self.mark_dirty();
        self.epoch = self.epoch.next();
    }

    /// Mark a computation pending, keeping the queued relayouts.
    pub(crate) fn mark_dirty(&mut self) {
        if let Pending::Clean = self.pending {
            self.pending = Pending::Dirty {
                relayout: Vec::new(),
            };
        }
    }

    /// Queue `parent` to be laid out again before the roots.
    pub(crate) fn queue_relayout(&mut self, parent: NodeId) {
        match &mut self.pending {
            Pending::Clean => {
                self.pending = Pending::Dirty {
                    relayout: vec![parent],
                };
            }
            Pending::Dirty { relayout } => relayout.push(parent),
        }
    }

    /// The last queued parent, if any; the state is clean once it is drained.
    pub(crate) fn relayout_pop(&mut self) -> Option<NodeId> {
        let Pending::Dirty { relayout } = &mut self.pending else {
            return None;
        };
        let id = relayout.pop();
        if id.is_none() {
            self.pending = Pending::Clean;
        }
        id
    }

    /// Drop the queued relayouts (a full rebuild lays every parent out anyway).
    pub(crate) fn clear_relayout(&mut self) {
        if let Pending::Dirty { relayout } = &mut self.pending {
            relayout.clear();
        }
    }

    /// Take the pending computation, leaving the state clean: `None` when there
    /// is nothing to do. The caller hands the queued parents to the computation.
    /// Geometry reports and scrolling can still require work when this is `None`.
    pub(crate) fn take_changed(&mut self) -> Option<Pending> {
        match std::mem::replace(&mut self.pending, Pending::Clean) {
            Pending::Clean => None,
            pending => Some(pending),
        }
    }
}
