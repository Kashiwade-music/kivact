//! Role: Borrowed-tree layout computation and derived geometry.
//!
//! Taffy computes flex and grid layout from an integration's borrowed
//! [`LayoutTree`] and [`LeafMeasure`]. The integration owns authoritative
//! identities, topology and resources; this crate owns derived columns,
//! measurement records, caches and rounded geometry observations.
//!
//! Keep [`NodeLayoutColumns`], [`LayoutState`] and the tree's slot mapping
//! in one allocation domain. Reset a slot before reuse, keep topology stable
//! during computation, and provide constant-time indexed child access.
//! Initialize an integration in this order:
//! 1. Start with default columns and state. The columns already include slot
//!    zero for the synthetic viewport; do not append another viewport slot.
//! 2. Call [`NodeLayoutColumns::push`] only for fresh structural slots, then
//!    [`NodeLayoutColumns::rebuild_style`] from each new live node's raw props.
//! 3. Call [`LayoutState::topology_changed`] after initial population, and after
//!    every parent/child link, root, overlay or fixed-membership edit. This
//!    initial notification is required even before the first computation.
//! 4. Consume [`LayoutState::take_changed`] when deciding whether to call
//!    [`NodeLayoutColumns::compute`], then consume the results from
//!    [`NodeLayoutColumns::drain_geometry_changes`].
//! 5. Call [`NodeLayoutColumns::reset`] before freeing or reusing a slot; reuse
//!    that existing slot rather than appending columns for it.
//!
//! Keep the same initialization and topology notifications when rebuilding
//! derived state. Default cursors and the state's epoch both start at zero;
//! omitting the initial notification can leave fixed-child counts cached as zero.
//!
//! Results are neutral; the integration decides rendering and accessibility.
//! Taffy types and caches remain private. Geometry/rebuild comparisons are
//! ordinary derived-state operations and do not expose mutable internals.
//!
//! Limits: no named grid lines, grid-template-areas, subgrid, order, block
//! or float layout. Absolute boxes use their parent's box; calc lengths
//! combine only percentages and pixels. Column-flex min/fit-content widths
//! measure their content unwrapped. Dirty ancestor containers revisit their
//! children; measurement-equivalent content changes can avoid a layout pass.
#![forbid(unsafe_code)]

mod calcs;
mod compare;
mod image;
mod placement;
mod style;
mod traversal;
mod tree;

pub use image::{image_content_size, style_known};
pub use placement::place_overlay;

use calcs::Calcs;
use kivact_wire::{LeafSize, MeasureRequest, Point, Rect, Size};
use std::cell::{Cell, RefCell};
use style::node::{NodeStyle, TStyle};

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

/// The structural role of a node in layout traversal; absolute style is independent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Participates in its parent's child list; absolute style remains independent.
    Normal,
    /// A fixed candidate, attached through the shared viewport when it has a parent.
    Fixed,
    /// An overlay root, excluded from its parent's layout child list.
    Overlay,
}

/// Borrowed authoritative topology and raw properties over reusable zero-based slots.
/// Logical identity zero is reserved for the synthetic viewport; it must not
/// identify an authoritative node. Live identity-to-slot and slot-to-identity
/// mappings must be mutual inverses. Parent and child links must agree, be
/// acyclic, and reference live allocated slots.
/// A slot is valid only while its node is live. Children are accessed directly,
/// without a converted slice, copied topology, or a second identity map.
/// During a computation all topology reads must be stable and child_at must be
/// constant-time indexed access. Integrations with a mutable editing order must
/// finish their ordinary packing barrier before creating this borrowed view.
pub trait LayoutTree {
    /// The main layout root, when attached.
    fn root(&self) -> Option<kivact_wire::NodeId>;
    /// Attached overlay roots in attachment order.
    fn overlays(&self) -> &[kivact_wire::NodeId];
    /// Fixed candidates in identity order; detached candidates remain present.
    fn fixed(&self) -> &std::collections::BTreeSet<kivact_wire::NodeId>;
    /// Resolve a live logical identity to its structural slot.
    fn index_of(&self, id: kivact_wire::NodeId) -> Option<usize>;
    /// The identity at a slot, or None for a vacant slot.
    fn id_at(&self, slot: usize) -> Option<kivact_wire::NodeId>;
    /// The authoritative parent of a live slot.
    fn parent(&self, slot: usize) -> Option<usize>;
    /// The number of unfiltered authoritative children of a live slot.
    fn child_count(&self, slot: usize) -> usize;
    /// A child's structural slot; raw_child must be below child_count.
    /// This is a constant-time lookup into the caller's stable packed order;
    /// it must not allocate or walk from the start for each index.
    fn child_at(&self, slot: usize, raw_child: usize) -> usize;
    /// Whether a live slot belongs in its parent's flow or another root.
    fn flow(&self, slot: usize) -> Flow;
    /// Raw layout properties and the raw hidden flag of a live slot.
    fn layout_props(&self, slot: usize) -> (Option<&kivact_wire::LayoutProps>, bool);
    /// The anchor whose current layout width floors this overlay, if requested.
    fn overlay_width_anchor(&self, slot: usize) -> Option<kivact_wire::NodeId>;
}

/// Measures content and resources without owning layout caches or retaining the callback.
/// During computation the paired columns are mutably borrowed. A measurement
/// callback must not query, mutate or reenter the active same-domain layout
/// columns or state (including geometry/layout reads, rebuilds or computation).
/// Such reentry violates the borrow contract and may panic. Record revalidation
/// also requires retained measurement records to stay unchanged until it returns.
pub trait LeafMeasure {
    /// Measure the logical node under logical-pixel constraints, including baseline.
    fn measure(&mut self, id: kivact_wire::NodeId, request: MeasureRequest) -> LeafSize;
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

/// Complete neutral geometry, opaque so cache and Taffy representations stay private.
/// Equality includes rounded/unrounded location, size, overflow rectangle,
/// scrollbars, border, padding and margin, and excludes traversal order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    rounded: Option<taffy::Layout>,
    unrounded: taffy::Layout,
}

/// One node-qualified difference between independently derived layout results.
#[derive(Clone, Debug)]
pub struct LayoutDifference {
    /// The logical node whose derived result differs.
    pub id: kivact_wire::NodeId,
    /// Geometry, constraint or measurement evidence, without cache handles.
    pub detail: String,
}
