#![deny(clippy::too_many_lines, clippy::disallowed_types)]
//! Role: Borrowed-tree layout computation and derived geometry.
//!
//! Taffy computes flex and grid layout from an integration's borrowed
//! [`LayoutTree`] and [`LeafMeasurer`]. The integration owns authoritative
//! identities, topology and resources; this crate owns derived columns,
//! measurement records, caches and rounded geometry observations.
//!
//! Reset a slot before reuse.
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
mod layout;
mod pass_state;
mod placement;
mod style;
mod traversal;
mod tree;

pub use borrowed_tree::{Flow, LayoutTree, LeafMeasurer};
pub(crate) use columns::NodeLayoutColumns;
pub(crate) use columns::{ChildCursor, Counted, NodeLayoutState, NodeLayoutValues};
pub use columns::{GeometryChange, GeometryChangeKind, GeometryChanges, NodeLayout};
pub use compare::{Geometry, LayoutDifference};
pub use image::{image_content_size, style_known};
pub use layout::Layout;
pub(crate) use pass_state::LayoutState;
pub use placement::{anchor_clear_size, place_overlay};
