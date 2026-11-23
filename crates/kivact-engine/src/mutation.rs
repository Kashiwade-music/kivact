//! Role: Counts client writes independently of derived model synchronization.

use kivact_types::{NodeId, Revision};
use std::collections::HashSet;

/// Client mutation totals and the last committed revision's operation counts.
#[derive(Clone, Copy, Debug, Default)]
pub struct WriteStats {
    /// Cumulative accessibility payloads constructed, excluding debug references.
    pub a11y_constructed: u64,
    /// Cumulative payloads sent to the window, including rejected attempts.
    pub a11y_published: u64,
    /// Cumulative nodes the accessibility identity enumeration visited, excluding debug references.
    pub a11y_enumerated: u64,
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
    /// The nodes created since the last commit (`Model::controlled_value` reads it).
    created: HashSet<NodeId>,
    /// The writes since the last commit.
    pending: u32,
}

impl Writes {
    pub(crate) fn created(&self, id: NodeId) -> bool {
        self.created.contains(&id)
    }
    pub(crate) fn create(&mut self, id: NodeId) {
        self.stats.creates += 1;
        self.created.insert(id);
        self.pending += 1;
    }
    pub(crate) fn field(&mut self) {
        self.stats.set_props += 1;
        self.pending += 1;
    }
    pub(crate) fn scene(&mut self) {
        self.stats.set_props += 1;
        self.pending += 1;
    }
    pub(crate) fn structural(&mut self) {
        self.stats.structural += 1;
        self.pending += 1;
    }
    pub(crate) fn command(&mut self) {
        self.pending += 1;
    }
    pub(crate) fn pending(&self) -> usize {
        self.pending as usize
    }
    /// Close the revision `revision`: `touched` nodes were written in it.
    pub(crate) fn commit(&mut self, revision: Revision, touched: usize) {
        let ops = self.pending();
        self.stats.batches += 1;
        self.stats.ops += ops as u64;
        self.stats.revision = revision.0;
        self.stats.last_ops = ops;
        self.stats.touched_last = touched;
        self.created.clear();
        self.pending = 0;
    }
}

/// What a typed write outdates, one bit each.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mutation(u16);

impl Mutation {
    /// Only the node itself is redrawn and exposed again.
    pub(crate) const NONE: Self = Self(0);
    /// The layout style is built again (and a layout pass runs).
    pub(crate) const STYLE: Self = Self(1);
    /// The node is measured again.
    pub(crate) const MEASURE: Self = Self(1 << 1);
    /// An inherited text style: the texts, text inputs and images below are measured or drawn again.
    pub(crate) const TEXT: Self = Self(1 << 2);
    /// Its transform or z-index may have moved it without a layout.
    pub(crate) const PLACED: Self = Self(1 << 3);
    /// The paint list is built again.
    pub(crate) const RESTACK: Self = Self(1 << 4);
    /// Its base paint: a state transition is retargeted and its place read.
    pub(crate) const PAINT: Self = Self(1 << 5);
    /// Its state styles: like `PAINT`.
    pub(crate) const STATES: Self = Self(1 << 6);
    /// A composition is committed under the old read-only policy first.
    pub(crate) const READ_ONLY: Self = Self(1 << 7);
    /// Inherited semantic state changes throughout the subtree.
    pub(crate) const SUBTREE: Self = Self(1 << 8);
    /// Only the node's own accessibility payload can change: its exposure,
    /// hierarchy, relations, regions and clipping stay (a text, a paint,
    /// state styles, a text style, an image, a placeholder).
    pub(crate) const PAYLOAD_ONLY: Self = Self(1 << 9);

    /// Whether any bit of `bits` is set.
    pub(crate) const fn has(self, bits: Self) -> bool {
        self.0 & bits.0 != 0
    }
    /// These bits and `other`'s.
    pub(crate) const fn and(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// These bits when `on`, else none.
    pub(crate) const fn when(self, on: bool) -> Self {
        if on { self } else { Self::NONE }
    }
    /// Add `bits`.
    pub(crate) fn insert(&mut self, bits: Self) {
        self.0 |= bits.0;
    }
}
