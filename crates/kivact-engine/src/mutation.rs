//! Role: Counts client writes independently of derived model synchronization.

use kivact_types::{NodeId, Revision};
use std::collections::HashSet;

/// Client mutation totals and the last committed revision's operation counts.
#[derive(Clone, Copy, Debug, Default)]
pub struct WriteStats {
    /// Cumulative accessibility payloads constructed, excluding debug references.
    #[cfg(feature = "testing")]
    pub a11y_constructed: u64,
    /// Cumulative payloads sent to the window, including rejected attempts.
    #[cfg(feature = "testing")]
    pub a11y_published: u64,
    pub creates: u64,
    pub set_props: u64,
    pub structural: u64,
    pub batches: u64,
    pub ops: u64,
    pub revision: u64,
    pub last_ops: usize,
    pub touched_last: usize,
}
#[derive(Default)]
pub(crate) struct Writes {
    pub(crate) stats: WriteStats,
    created: HashSet<NodeId>,
    updated: HashSet<NodeId>,
    touched: HashSet<NodeId>,
    sparse_touched_revisions: u8,
    commands: usize,
}
impl Writes {
    pub(crate) fn created(&self, id: NodeId) -> bool {
        self.created.contains(&id)
    }
    pub(crate) fn create(&mut self, id: NodeId) {
        self.stats.creates += 1;
        self.created.insert(id);
        self.touched.insert(id);
    }
    pub(crate) fn field(&mut self, id: NodeId) {
        self.stats.set_props += 1;
        if !self.created.contains(&id) {
            self.updated.insert(id);
        }
        self.touched.insert(id);
    }
    pub(crate) fn scene(&mut self, id: NodeId) {
        self.stats.set_props += 1;
        self.command(Some(id));
    }
    pub(crate) fn structural(&mut self, id: NodeId) {
        self.stats.structural += 1;
        self.command(Some(id));
    }
    pub(crate) fn command(&mut self, id: Option<NodeId>) {
        self.commands += 1;
        if let Some(id) = id {
            self.touched.insert(id);
        }
    }
    pub(crate) fn pending(&self) -> usize {
        self.created.len() + self.updated.len() + self.commands
    }
    pub(crate) fn commit(&mut self, revision: Revision) {
        let ops = self.pending();
        self.stats.batches += 1;
        self.stats.ops += ops as u64;
        self.stats.revision = revision.0;
        self.stats.last_ops = ops;
        self.stats.touched_last = self.touched.len();
        self.created.clear();
        self.updated.clear();
        // Clearing a large retained table costs more even for one touched node.
        // Keep dense bursts reusable: shrink only after four revisions below
        // one eighth of capacity, and leave small tables (up to 256 slots) alone.
        let sparse = self.touched.capacity() > 256
            && self.touched.capacity() > self.stats.touched_last.saturating_mul(8);
        self.sparse_touched_revisions = if sparse {
            self.sparse_touched_revisions + 1
        } else {
            0
        };
        if self.sparse_touched_revisions == 4 {
            self.touched = HashSet::with_capacity(self.stats.touched_last);
            self.sparse_touched_revisions = 0;
        } else {
            self.touched.clear();
        }
        self.commands = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mutation {
    /// The layout style is built again (and a layout pass runs).
    pub(super) style: bool,
    /// The node is measured again.
    pub(super) measure: bool,
    /// An inherited text style: the texts, text inputs and images below
    /// are measured or drawn again.
    pub(super) text: bool,
    /// Its transform or z-index may have moved it without a layout.
    pub(super) placed: bool,
    /// The paint list is built again.
    pub(super) restack: bool,
    /// Its base paint: a state transition is retargeted and its place read.
    pub(super) paint: bool,
    /// Its state styles: like `paint`.
    pub(super) states: bool,
    /// A composition is committed under the old read-only policy first.
    pub(super) read_only: bool,
    /// Inherited semantic state changes throughout the subtree.
    pub(super) subtree: bool,
    /// Existing plain text content or native document/session only.
    pub(super) local_text: bool,
}
impl Mutation {
    /// Only the node itself is redrawn and exposed again.
    pub(crate) const NONE: Self = Self {
        style: false,
        measure: false,
        text: false,
        placed: false,
        restack: false,
        paint: false,
        states: false,
        read_only: false,
        subtree: false,
        local_text: false,
    };
    pub(crate) const LOCAL_TEXT: Self = Self {
        local_text: true,
        ..Self::NONE
    };
    pub(crate) const SUBTREE: Self = Self {
        subtree: true,
        ..Self::NONE
    };
    pub(crate) const STYLE: Self = Self {
        style: true,
        ..Self::NONE
    };
    pub(crate) const MEASURE: Self = Self {
        measure: true,
        ..Self::NONE
    };
    pub(crate) const TEXT: Self = Self {
        text: true,
        ..Self::NONE
    };
    pub(crate) const PLACED: Self = Self {
        placed: true,
        ..Self::NONE
    };
    pub(crate) const RESTACK: Self = Self {
        restack: true,
        ..Self::NONE
    };
    pub(crate) const PAINT: Self = Self {
        paint: true,
        ..Self::NONE
    };
    pub(crate) const STATES: Self = Self {
        states: true,
        ..Self::NONE
    };
    pub(crate) const READ_ONLY: Self = Self {
        read_only: true,
        ..Self::NONE
    };

    /// Both mutations' flags.
    pub(crate) const fn and(self, other: Self) -> Self {
        Self {
            style: self.style || other.style,
            measure: self.measure || other.measure,
            text: self.text || other.text,
            placed: self.placed || other.placed,
            restack: self.restack || other.restack,
            paint: self.paint || other.paint,
            states: self.states || other.states,
            read_only: self.read_only || other.read_only,
            subtree: self.subtree || other.subtree,
            local_text: self.local_text || other.local_text,
        }
    }

    /// These flags when `on`, else none.
    pub(crate) const fn when(self, on: bool) -> Self {
        if on { self } else { Self::NONE }
    }
}
