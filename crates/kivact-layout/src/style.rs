//! Role: Taffy style conversion from layout properties.
//!
//! Taffy style of the `wire` [`LayoutProps`]: the one conversion the layout
//! pass on retained nodes lays out with.
//!
//! Invariants:
//! - Taffy types remain internal to this crate. Shared style declarations
//!   live in `layout` and `style::node`, apart from this conversion and computation.
//! - A calc length becomes a handle owned by the retained node
//!   ([`CalcAlloc`]), which taffy resolves against the same basis as a
//!   percentage there.
//! - An unset `justify` (or `align_content`) stays unset: taffy's `normal`,
//!   the start in a flex box and stretched auto tracks in a grid.
//! - A content size keyword (`MinContent`, `MaxContent`, `FitContent`) is
//!   a size only as a width, a height or a basis (and in grid tracks).
//!   Elsewhere it is `auto` where the property takes `auto` (margins,
//!   insets, minimum and maximum sizes) and 0 where it does not (padding,
//!   gaps).
//! - A sticky box is laid out in flow with no insets; its integration
//!   applies the sticky offset afterwards. A fixed box is an absolute
//!   child of the viewport.
//! - The border's widths are taffy's border (a width that is not finite
//!   or below zero is 0): the content sits inside the border and the
//!   padding, and a size is the border box's (taffy's default
//!   `box-sizing: border-box`).

mod convert;
pub(crate) mod node;

pub(crate) use convert::taffy_style;
