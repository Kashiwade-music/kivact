//! Role: Borrowed-tree layout computation and derived geometry.
//!
//! Taffy computes flex and grid layout from an integration's borrowed
//! [`LayoutTree`] and [`LeafMeasurer`]. The integration owns authoritative
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
//! 3. Call [`LayoutState::topology_changed`] after every parent/child link, root,
//!    overlay or fixed-membership edit.
//! 4. Consume [`LayoutState::take_changed`] when deciding whether to call
//!    [`NodeLayoutColumns::compute`], then consume the results from
//!    [`NodeLayoutColumns::drain_geometry_changes`].
//! 5. Call [`NodeLayoutColumns::reset`] before freeing or reusing a slot; reuse
//!    that existing slot rather than appending columns for it.
//!
//! Keep the same initialization and topology notifications when rebuilding
//! derived state. A new state has its first computation pending and counts
//! every cursor on first use.
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

mod borrowed_tree;
mod calcs;
mod columns;
mod compare;
mod image;
mod pass_state;
mod placement;
mod style;
mod traversal;
mod tree;

pub use borrowed_tree::{Flow, LayoutTree, LeafMeasurer};
pub(crate) use columns::{ChildCursor, Counted, NodeLayoutState, NodeLayoutValues};
pub use columns::{
    GeometryChange, GeometryChangeKind, GeometryChanges, NodeLayout, NodeLayoutColumns,
};
pub use compare::{Geometry, LayoutDifference};
pub use image::{image_content_size, style_known};
pub use pass_state::LayoutState;
pub use placement::{anchor_clear_size, place_overlay};
