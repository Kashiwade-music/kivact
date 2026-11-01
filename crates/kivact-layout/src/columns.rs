//! Role: Per-node layout columns: results, measure records, cursors and geometry changes.

use crate::calcs::Calcs;
use crate::style::node::NodeStyle;
use kivact_wire::{LeafSize, MeasureRequest, Point, Rect, Size};
use std::cell::{Cell, RefCell};

/// The computed layout of one node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NodeLayout {
    /// Border-box origin relative to the parent's border box (unscrolled).
    pub location: Point,
    /// Border-box size.
    pub size: Size,
    /// Size of the content (what a scroll container scrolls over): the
    /// border box grown by how far it can scroll (taffy's scrollable
    /// overflow), never smaller than the box.
    pub content_size: Size,
    /// The content box (inside border and padding), relative to the node.
    pub content_box: Rect,
}

/// What a leaf measured to, by request: the last 16 requests (`sizes`);
/// `incomplete` once an older one was dropped, so the record no longer
/// covers every request the last computation made (see
/// [`NodeLayoutColumns::remeasures_same`]). An edit that invalidates the leaf's layout
/// clears it; nothing else measures a leaf again, frame after frame.
#[derive(Clone, Debug, Default)]
pub(crate) struct Record {
    pub(crate) sizes: Vec<(MeasureRequest, LeafSize)>,
    pub(crate) incomplete: bool,
}

/// A disposable filtered index cursor; never owns links or child IDs.
#[derive(Clone, Debug, Default)]
pub(crate) struct ChildCursor {
    pub(crate) epoch: Cell<u64>,
    pub(crate) count: Cell<usize>,
    pub(crate) unfiltered: Cell<bool>,
    pub(crate) at: Cell<Option<(usize, usize)>>,
}

/// Derived per-slot measurement records and calc ownership; never authoritative props.
#[derive(Clone, Debug, Default)]
pub(crate) struct NodeLayoutState {
    pub(crate) calcs: RefCell<Vec<u32>>,
    pub(crate) measured: RefCell<Record>,
}

/// Style, caches and optional geometry borrowed for one root computation.
#[derive(Clone, Debug)]
pub(crate) struct NodeLayoutValues {
    pub(crate) styles: Vec<NodeStyle>,
    pub(crate) caches: Vec<taffy::Cache>,
    pub(crate) unrounded: Vec<taffy::Layout>,
    pub(crate) final_layout: Vec<Option<taffy::Layout>>,
    /// Rounded geometry changes awaiting consumption by the integration.
    pub(crate) geometry_changed: Vec<GeometryChange>,
    /// The inputs of the node's last full layout (taffy's `PerformLayout`),
    /// which lay a parent of an absolute box out again on its own (`tree`).
    pub(crate) performed: Vec<Option<taffy::LayoutInput>>,
}

impl Default for NodeLayoutValues {
    fn default() -> Self {
        Self {
            styles: vec![NodeStyle::default()],
            caches: vec![taffy::Cache::new()],
            unrounded: vec![taffy::Layout::new()],
            final_layout: vec![None],
            geometry_changed: Vec::new(),
            performed: vec![None],
        }
    }
}

/// Layout components of the primary node store, sharing its stable positions.
/// Position zero is the viewport; no identity map or topology is stored here.
///
/// One column per component: the style derived from the layout props, the
/// taffy cache, the unrounded geometry, the rounded layout (`None` until
/// laid out), the last full layout's inputs, and the child traversal
/// cursor. A node's entry is at its store
/// position plus one; position zero is the shared viewport the fixed boxes
/// are laid out in. There is no layout-only id table, no copy of the
/// links and no replay of layout operations.
///
/// Each instance belongs to one structural slot-allocation domain. Allocate a
/// column slot when the matching tree slot is created; reset it before removal,
/// replacement or reuse. All tree-taking operations require that same domain:
/// a live identity must resolve to the slot whose derived state is held here.
/// Clone columns together with their tree/state to obtain an independent domain
/// with the same mapping; unrelated trees or reordered slot allocations are not
/// interchangeable. Slots in this API are zero-based, excluding the viewport.
#[derive(Clone, Debug)]
pub struct NodeLayoutColumns {
    pub(crate) cursors: Vec<ChildCursor>,
    pub(crate) states: Vec<NodeLayoutState>,
    pub(crate) calcs: RefCell<Calcs>,
    pub(crate) values: RefCell<NodeLayoutValues>,
}

impl Default for NodeLayoutColumns {
    fn default() -> Self {
        Self {
            cursors: vec![ChildCursor::default()],
            states: vec![NodeLayoutState::default()],
            calcs: RefCell::new(Calcs::default()),
            values: RefCell::new(NodeLayoutValues::default()),
        }
    }
}

impl NodeLayoutColumns {
    /// Append one fresh structural node slot to every derived column.
    /// Default columns already contain the synthetic viewport at slot zero.
    /// The new slot must match the next newly allocated slot of this instance's
    /// tree domain; reusing an existing reset slot does not append columns.
    pub fn push(&mut self) {
        self.cursors.push(ChildCursor::default());
        self.states.push(NodeLayoutState::default());
        let values = self.values.get_mut();
        values.styles.push(NodeStyle::default());
        values.caches.push(taffy::Cache::new());
        values.unrounded.push(taffy::Layout::new());
        values.final_layout.push(None);
        values.performed.push(None);
    }

    /// Release calcs and reset every derived component before a slot is reused.
    /// `slot` must be a zero-based allocated slot in this instance's domain,
    /// still owned by the node being removed or replaced. Reset before publishing
    /// its availability for reuse; reset does not change authoritative links.
    pub fn reset(&mut self, slot: usize) {
        let index = slot + 1;
        self.calcs
            .get_mut()
            .free
            .extend(std::mem::take(self.states[index].calcs.get_mut()));
        self.states[index] = NodeLayoutState::default();
        self.cursors[index] = ChildCursor::default();
        let values = self.values.get_mut();
        values.styles[index] = NodeStyle::default();
        values.caches[index] = taffy::Cache::new();
        values.unrounded[index] = taffy::Layout::new();
        values.final_layout[index] = None;
        values.performed[index] = None;
    }
}

impl NodeLayoutColumns {
    /// Drain neutral changes once; dropping the drain discards unread changes.
    /// Consume after computation and before the next compute/reset/reuse of the
    /// paired tree domain. The drain keeps an exclusive derived-values borrow;
    /// do not reenter geometry/computation operations while it is alive.
    pub fn drain_geometry_changes(&self) -> GeometryChanges<'_> {
        GeometryChanges {
            values: self.values.borrow_mut(),
            at: 0,
        }
    }
}

/// Whether rounded local geometry changed or only the parent's placement changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryChangeKind {
    /// Initial layout or a change to any rounded non-location geometry field.
    Local,
    /// Only rounded location changed; every local geometry field is unchanged.
    PositionOnly,
}

/// A rounded geometry observation; integrations decide paint/accessibility effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeometryChange {
    /// The logical identity, never a reusable slot.
    pub id: kivact_wire::NodeId,
    /// Local includes initial layout; order bookkeeping is excluded.
    pub kind: GeometryChangeKind,
}

/// An allocation-free drain retaining the columns' event-buffer capacity.
pub struct GeometryChanges<'a> {
    values: std::cell::RefMut<'a, NodeLayoutValues>,
    at: usize,
}

impl Iterator for GeometryChanges<'_> {
    type Item = GeometryChange;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.values.geometry_changed.get(self.at).copied()?;
        self.at += 1;
        Some(value)
    }
}

impl Drop for GeometryChanges<'_> {
    fn drop(&mut self) {
        self.values.geometry_changed.clear();
    }
}
