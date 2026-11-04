//! Role: Pending invalidation: one dirty record per node and the global flags.
//!
//! Writes, input and layout record what they outdated in [`Changes`];
//! `Model::sync` (`sync`) is the only dispatcher of that record into
//! derived work. A typed property write states what it outdates as a
//! [`Mutation`] (`host::write::typed` names one for every property), and
//! `Model::set_prop` (`apply`) records it.
//!
//! What is pending for one node is one [`Dirty`] bit set in one map; what
//! is pending for the whole tree is a flag. A bit means what the set it
//! replaced meant: `WRITTEN` (its own props, scenes or children were
//! written), `CHILDREN` (its list of children, or how one of them is
//! drawn among its siblings, changed: its child index is dropped; set only by
//! [`Changes::regroup`], which also sets `WRITTEN`), `REDRAW` (its drawing changed without a write: focus, hover or
//! pressed flipped, a text input's value, selection or composition moved,
//! an ancestor's text style changed, a layout moved its box), `MEASURE`
//! (its measured content changed), `LAYOUT_STYLE` (its layout style is
//! built again), `LAYOUT_CHAIN` (its layout cache and its ancestors' are
//! outdated), `SCROLLED` (a scroll container's offset changed, no box was
//! laid out again) and `STUCK` (a sticky box's shift changed).
//!
//! Invariants:
//! - Only the model's modules write the record directly; layout, paint
//!   and text geometry record through the named methods below.
//! - A remeasured node always asks for a layout pass.
//! - A mutation has no default: every property names its flags.
//! - A node has at most one entry, and an entry never holds no bit: a
//!   taken bit that was the last one removes the entry, and a removed node
//!   is forgotten whole ([`Changes::forget`]).
//! - `sync` takes `LAYOUT_STYLE` before `LAYOUT_CHAIN`; `MEASURE` is the one
//!   bit that survives a sync (until a layout pass takes it).
//! - Positions are one [`Geometry`]. `sync` takes what it dispatches with
//!   [`Changes::dispatch`], which leaves `MEASURE`, `needs_layout`,
//!   `layout_changed` and `viewport_chain` for the layout pass.

use std::collections::{HashMap, HashSet};

use kivact_types::NodeId;

/// The kinds of per-node pending work, as a small bit set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Dirty(u16);

impl Dirty {
    /// Its own props, scenes or children were written.
    pub(crate) const WRITTEN: Self = Self(1);
    /// Its drawing changed without a write of its own.
    pub(crate) const REDRAW: Self = Self(1 << 1);
    /// Its children changed (attached, removed, or sticky or fixed anew).
    const CHILDREN: Self = Self(1 << 7);
    /// Its measured content changed.
    pub(crate) const MEASURE: Self = Self(1 << 2);
    /// Its layout style is built again.
    pub(crate) const LAYOUT_STYLE: Self = Self(1 << 3);
    /// Its layout cache and its ancestors' are outdated.
    pub(crate) const LAYOUT_CHAIN: Self = Self(1 << 4);
    /// Its content moved: a scroll container's offset changed.
    pub(crate) const SCROLLED: Self = Self(1 << 5);
    /// Its sticky shift changed (it and its subtree moved).
    pub(crate) const STUCK: Self = Self(1 << 6);

    /// Proven content/session-only writes; mixed unknown changes override this.
    pub(crate) const A11Y_LOCAL_TEXT: Self = Self(1 << 8);
    /// An unclassified or metadata-affecting change on this same node.
    pub(crate) const A11Y_METADATA: Self = Self(1 << 9);

    /// Whether every bit of `bits` is set.
    pub(crate) const fn contains(self, bits: Self) -> bool {
        self.0 & bits.0 == bits.0
    }

    /// Whether any bit of `bits` is set.
    pub(crate) const fn intersects(self, bits: Self) -> bool {
        self.0 & bits.0 != 0
    }

    /// Set the bits of `bits`.
    pub(crate) const fn insert(&mut self, bits: Self) {
        self.0 |= bits.0;
    }

    /// Clear the bits of `bits`.
    const fn remove(&mut self, bits: Self) {
        self.0 &= !bits.0;
    }

    /// Both sets' bits.
    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// How far boxes may have moved since the last sync.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Geometry {
    /// No box moved.
    #[default]
    Kept,
    /// Positions may have moved without a layout computation (a scroll offset, a
    /// sticky shift, a moving transform, a placement): positions are drawn again.
    Moved,
    /// The layout was computed: any box may have moved or resized.
    LaidOut,
}

/// Pending work; only sync dispatches visual invalidation.
#[derive(Default)]
pub(crate) struct Changes {
    /// What is pending for each node (see the module docs).
    pub(super) dirty: HashMap<NodeId, Dirty>,
    /// How far boxes may have moved (a host caching positions draws them
    /// again).
    pub(super) geometry: Geometry,
    pub(super) needs_layout: bool,
    pub(super) paint: bool,
    pub(super) restack: bool,
    pub(super) fonts: bool,
    pub(super) a11y_structure: bool,
    pub(super) a11y_subtrees: HashSet<NodeId>,
    pub(super) a11y_removed: HashSet<NodeId>,
    pub(super) a11y_focus: bool,
    pub(super) layout_changed: bool,
    /// The viewport's layout cache is outdated (a fixed box came or went).
    pub(super) viewport_chain: bool,
}

/// What one sync dispatches into derived work (`Changes::dispatch`).
pub(crate) struct Dispatched {
    dirty: HashMap<NodeId, Dirty>,
    pub(crate) geometry: Geometry,
    pub(crate) paint: bool,
    pub(crate) restack: bool,
    pub(crate) fonts: bool,
    pub(crate) a11y_structure: bool,
    pub(crate) a11y_subtrees: HashSet<NodeId>,
    pub(crate) a11y_removed: HashSet<NodeId>,
    pub(crate) a11y_focus: bool,
}

impl Dispatched {
    /// Whether any node has any bit of `bits`.
    pub(crate) fn any(&self, bits: Dirty) -> bool {
        self.dirty.values().any(|d| d.intersects(bits))
    }

    /// The nodes with any bit of `bits`.
    pub(crate) fn marked(&self, bits: Dirty) -> impl Iterator<Item = NodeId> + '_ {
        self.dirty
            .iter()
            .filter(move |(_, d)| d.intersects(bits))
            .map(|(id, _)| *id)
    }

    /// What is pending for `id`.
    pub(crate) fn flags(&self, id: NodeId) -> Dirty {
        self.dirty.get(&id).copied().unwrap_or_default()
    }

    /// Whether the layout was computed.
    pub(crate) fn was_laid_out(&self) -> bool {
        self.geometry == Geometry::LaidOut
    }

    /// The nodes whose children changed.
    pub(crate) fn regrouped(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.marked(Dirty::CHILDREN)
    }

    /// The nodes whose drawing changed, with or without a write, and the
    /// scroll containers whose content moved.
    pub(crate) fn redrawn(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.marked(Dirty::WRITTEN.union(Dirty::REDRAW).union(Dirty::SCROLLED))
    }
}

impl Changes {
    /// The first record: everything is laid out once.
    pub(super) fn initial() -> Self {
        Self {
            needs_layout: true,
            ..Self::default()
        }
    }

    /// Record `bits` for `id`.
    pub(super) fn mark(&mut self, id: NodeId, bits: Dirty) {
        self.dirty.entry(id).or_default().insert(bits);
    }

    /// Positions may have moved (no layout computation).
    pub(crate) fn moved(&mut self) {
        self.geometry = self.geometry.max(Geometry::Moved);
    }

    /// The list of children of `parent`, or how one of them is drawn among its
    /// siblings, changed: it is written, and its child index is dropped.
    pub(crate) fn regroup(&mut self, parent: NodeId) {
        self.mark(parent, Dirty::WRITTEN.union(Dirty::CHILDREN));
    }

    /// The viewport's layout cache is outdated (a fixed box came or went).
    pub(crate) fn outdate_viewport(&mut self) {
        self.viewport_chain = true;
    }

    /// Take what one sync dispatches. `MEASURE` bits, `needs_layout`,
    /// `layout_changed` and `viewport_chain` stay: the layout pass takes them.
    pub(super) fn dispatch(&mut self) -> Dispatched {
        let mut dirty = std::mem::take(&mut self.dirty);
        dirty.retain(|id, d| {
            if d.contains(Dirty::MEASURE) {
                self.dirty.insert(*id, Dirty::MEASURE);
                d.remove(Dirty::MEASURE);
            }
            !d.is_empty()
        });
        Dispatched {
            dirty,
            geometry: std::mem::take(&mut self.geometry),
            paint: std::mem::take(&mut self.paint),
            restack: std::mem::take(&mut self.restack),
            fonts: std::mem::take(&mut self.fonts),
            a11y_structure: std::mem::take(&mut self.a11y_structure),
            a11y_subtrees: std::mem::take(&mut self.a11y_subtrees),
            a11y_removed: std::mem::take(&mut self.a11y_removed),
            a11y_focus: std::mem::take(&mut self.a11y_focus),
        }
    }

    /// The nodes with any bit of `bits`.
    pub(super) fn marked(&self, bits: Dirty) -> impl Iterator<Item = NodeId> + '_ {
        self.dirty
            .iter()
            .filter(move |(_, d)| d.intersects(bits))
            .map(|(id, _)| *id)
    }

    /// The nodes with the bit `bit`, which is cleared for them.
    pub(super) fn take(&mut self, bit: Dirty) -> Vec<NodeId> {
        let mut taken = Vec::new();
        self.dirty.retain(|id, d| {
            if d.contains(bit) {
                taken.push(*id);
                d.remove(bit);
            }
            !d.is_empty()
        });
        taken
    }

    /// `id` is gone: nothing is pending for it.
    pub(super) fn forget(&mut self, id: NodeId) {
        self.dirty.remove(&id);
    }

    /// Start a layout pass: the nodes to measure again, or `None` when the
    /// pass has nothing to do.
    pub(crate) fn begin_layout(&mut self) -> Option<Vec<NodeId>> {
        if !std::mem::take(&mut self.needs_layout) {
            return None;
        }
        Some(self.take(Dirty::MEASURE))
    }

    /// The layout pass computed the tree: any box may have moved.
    pub(crate) fn laid_out(&mut self, geometry_changed: impl Iterator<Item = NodeId>) {
        for id in geometry_changed {
            self.mark(id, Dirty::REDRAW);
        }
        self.geometry = Geometry::LaidOut;
    }

    /// The scroll container `id` moved its content.
    pub(crate) fn scrolled(&mut self, id: NodeId) {
        self.needs_layout = true;
        self.moved();
        self.mark(id, Dirty::SCROLLED);
    }

    /// Settling scroll offsets left nothing for another pass.
    pub(crate) fn settled(&mut self) {
        self.needs_layout = false;
    }

    /// The sticky box `id` moved with its scrollport.
    pub(crate) fn stuck(&mut self, id: NodeId) {
        self.moved();
        self.mark(id, Dirty::STUCK);
    }

    /// Whether the last layout pass computed the tree.
    pub(crate) fn was_laid_out(&self) -> bool {
        self.geometry == Geometry::LaidOut
    }

    /// The nodes a write touched (their own props, scenes or children).
    pub(crate) fn touched(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.marked(Dirty::WRITTEN)
    }
}
