//! Role: The borrowed tree and leaf measurer a caller lends to a layout pass.

use kivact_wire::{LeafSize, MeasureRequest};

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
/// Live identity-to-slot and slot-to-identity
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
    /// The opted-in viewport fitting policy for this overlay.
    fn overlay_fit(&self, slot: usize) -> Option<kivact_wire::OverlayFit>;
}

/// Measures content and resources without owning layout caches or retaining the callback.
/// The columns are lent mutably for the whole computation, so a measurer cannot
/// reach them. Record revalidation requires retained measurement records to
/// stay unchanged until it returns.
pub trait LeafMeasurer {
    /// Measure the logical node under logical-pixel constraints, including baseline.
    fn measure(&mut self, id: kivact_wire::NodeId, request: MeasureRequest) -> LeafSize;
}
