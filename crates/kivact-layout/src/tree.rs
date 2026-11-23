//! Role: Computes retained layouts from borrowed topology and leaf measurements.
//!
//! The layout pass implements taffy's custom tree traits on `LayoutPass`,
//! which exists only while one root is computed: it shares the borrow of
//! the `LayoutTree` and borrows `LeafMeasurer` and the layout columns mutably,
//! and no copy of the tree or shared measuring context outlives it. Each root
//! computation splits the columns once: the styles as a shared slice, the
//! caches and the geometry as mutable slices, indexed directly.
//! The tree parameter preserves its concrete type through traversal so Taffy
//! can specialize at the caller.
//!
//! Invariants:
//! - The root fills the window, then each attached overlay is a root of
//!   its own (in the order attached), then the fixed boxes, laid out as
//!   children of the shared viewport (position zero) in `NodeId` order;
//!   consumers retain their own presentation order independently.
//! - A leaf's measurements are recorded by request (`Record`), in the f32
//!   the layout reads, and taken from the record before the leaf is measured.
//! - A leaf's first baseline is placed inside its padding and border:
//!   from the content top, or from the content centre (a single-line text
//!   input's).
//! - Invalidating a node clears its record and the caches up the chain
//!   until a cache already empty, an overlay (a root of its own), a fixed
//!   box (which clears the viewport's instead), or an absolute box. An
//!   absolute box takes no part in its parent's size (taffy places it after
//!   the in-flow items), so its parent is queued instead
//!   (`LayoutState::relayout`): before the roots, the parent is laid out
//!   again alone with the inputs of its last full layout (the `performed`
//!   column), and its own chain is cleared only when that answers another
//!   size, baseline or reach of its overflow than its cache held. A box that becomes or stops being absolute
//!   clears the chain through itself (its parent's intrinsic sizes change).
//! - Layouts are rounded to whole pixels (`round_changed`: taffy's
//!   `round_layout` only where something changed);
//!   `NodeLayoutColumns::layout_at` reads a node's by its position, and only a
//!   lookup by `NodeId` goes through the store's index.
//! - A box whose own lengths are absolute is cached without its parent's size
//!   (`relative`).
//! - Every node is positioned (`Relative` or `Absolute`), so an absolute
//!   box's containing block is its layout parent: the parent places it in its
//!   own full layout (`root::place_out_of_flow`), and a root is laid out at
//!   the origin as taffy 0.14 did (`root::layout_root`).

mod compute;
mod invalidation;
mod partial;
mod pass;
mod readout;
mod root;
mod rounding;
