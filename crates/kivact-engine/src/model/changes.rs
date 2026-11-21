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
//!   [`Changes::dispatch`], which leaves `MEASURE` and `LayoutReasons` for the layout pass.

use std::collections::HashMap;

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
    /// A text input's caret, text, size or style changed: it is scrolled to keep the caret in view (`Model::scroll_inputs`). Kept by `dispatch` until taken.
    pub(crate) const CARET: Self = Self(1 << 8);
    /// The geometry reports at or below it are due (`Model::report_geometry`). Kept by `dispatch` until taken.
    pub(crate) const REPORT: Self = Self(1 << 9);

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

/// Pending reasons for a layout pass and its invalidations.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LayoutReasons(u8);
impl LayoutReasons {
    pub(crate) const PASS: Self = Self(1);
    pub(crate) const STYLE: Self = Self(2);
    pub(crate) const VIEWPORT: Self = Self(4);
    pub(crate) fn insert(&mut self, reason: Self) {
        self.0 |= reason.0;
    }
    pub(crate) fn contains(self, reason: Self) -> bool {
        self.0 & reason.0 != 0
    }
    pub(crate) fn take(&mut self, reason: Self) -> bool {
        let contained = self.contains(reason);
        self.0 &= !reason.0;
        contained
    }
    pub(crate) fn bits(self) -> u8 {
        self.0
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

// xtask-allow(state): legacy: paint, restack, fonts; raised by Model::invalidate_paint (host environment and asset events) and by image and layout-motion steps; restack and fonts
/// Pending work; only sync dispatches visual invalidation.
#[derive(Default)]
pub(crate) struct Changes {
    /// What is pending for each node (see the module docs).
    pub(super) dirty: HashMap<NodeId, Dirty>,
    /// How far boxes may have moved (a host caching positions draws them
    /// again).
    pub(super) geometry: Geometry,
    pub(super) layout: LayoutReasons,
    pub(super) paint: bool,
    pub(super) restack: bool,
    pub(super) fonts: bool,
}

/// What one sync dispatches into derived work (`Changes::dispatch`).
pub(crate) struct Dispatched {
    dirty: HashMap<NodeId, Dirty>,
    pub(crate) geometry: Geometry,
    pub(crate) paint: bool,
    pub(crate) restack: bool,
    pub(crate) fonts: bool,
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
            layout: LayoutReasons::PASS,
            ..Self::default()
        }
    }

    /// Whether nothing is pending: no node is marked, no layout, paint,
    /// restack or font work waits and no box moved.
    pub(crate) fn is_clean(&self) -> bool {
        self.dirty.is_empty()
            && self.geometry == Geometry::Kept
            && self.layout.bits() == 0
            && !self.paint
            && !self.restack
            && !self.fonts
    }

    /// Record `bits` for `id`.
    pub(super) fn mark(&mut self, id: NodeId, bits: Dirty) {
        self.dirty.entry(id).or_default().insert(bits);
    }

    /// Positions may have moved (no layout computation).
    pub(crate) fn moved(&mut self) {
        self.geometry = self.geometry.max(Geometry::Moved);
    }

    /// `id` moved without a layout computation (a transform, a placement,
    /// a scroll offset): positions are redrawn and its reports are due.
    pub(crate) fn moved_node(&mut self, id: NodeId) {
        self.moved();
        self.mark(id, Dirty::REPORT);
    }

    /// The list of children of `parent`, or how one of them is drawn among its
    /// siblings, changed: it is written, and its child index is dropped.
    pub(crate) fn regroup(&mut self, parent: NodeId) {
        self.mark(parent, Dirty::WRITTEN.union(Dirty::CHILDREN));
    }

    /// The viewport's layout cache is outdated (a fixed box came or went).
    pub(crate) fn outdate_viewport(&mut self) {
        self.layout.insert(LayoutReasons::VIEWPORT);
    }

    /// Take what one sync dispatches. Measurement bits and layout reasons stay.
    pub(super) fn dispatch(&mut self) -> Dispatched {
        let mut dirty = std::mem::take(&mut self.dirty);
        let kept_bits = Dirty::MEASURE.union(Dirty::CARET).union(Dirty::REPORT);
        dirty.retain(|id, d| {
            let kept = Dirty(d.0 & kept_bits.0);
            if !kept.is_empty() {
                self.dirty.insert(*id, kept);
                d.remove(kept);
            }
            !d.is_empty()
        });
        Dispatched {
            dirty,
            geometry: std::mem::take(&mut self.geometry),
            paint: std::mem::take(&mut self.paint),
            restack: std::mem::take(&mut self.restack),
            fonts: std::mem::take(&mut self.fonts),
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

    /// Start a layout pass: the nodes to measure again (their reports are
    /// due), or `None` when the pass has nothing to do.
    pub(crate) fn begin_layout(&mut self) -> Option<Vec<NodeId>> {
        if !self.layout.take(LayoutReasons::PASS) {
            return None;
        }
        let measured = self.take(Dirty::MEASURE);
        for id in &measured {
            self.mark(*id, Dirty::REPORT);
        }
        Some(measured)
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
        self.layout.insert(LayoutReasons::PASS);
        self.mark(id, Dirty::SCROLLED.union(Dirty::REPORT));
    }

    /// Settling scroll offsets left nothing for another pass.
    pub(crate) fn settled(&mut self) {
        self.layout.take(LayoutReasons::PASS);
    }

    /// The sticky box `id` moved with its scrollport.
    pub(crate) fn stuck(&mut self, id: NodeId) {
        self.moved();
        self.mark(id, Dirty::STUCK.union(Dirty::REPORT));
    }

    /// Whether the last layout pass computed the tree (read by the debug layout lint).
    #[cfg(debug_assertions)]
    pub(crate) fn was_laid_out(&self) -> bool {
        self.geometry == Geometry::LaidOut
    }

    /// The nodes a write touched (their own props, scenes or children).
    pub(crate) fn touched(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.marked(Dirty::WRITTEN)
    }
}
