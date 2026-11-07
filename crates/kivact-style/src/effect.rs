//! Role: Shapes, shadows, filters and blending, and their engine forms.
//!
//! Shapes, shadows, filters and blending: the effects vocabulary of
//! `Style` (and of custom drawing), and how a shape and a filter become the
//! engine's.

mod filter;
mod filter_graph;
mod shadow;
mod shape;

pub use filter::Filter;
pub(crate) use filter::{filter_ops, filter_problems};
pub use filter_graph::{FilterGraph, FilterInput};
pub use kivact_types::{
    Channel, CompositeOp, EdgeMode, Light, MorphologyOp, TransferFn, TurbulenceKind,
};
pub use shadow::Shadow;
pub use shape::Shape;
pub(crate) use shape::{basic, clip_shape};

/// How a border's line is drawn (CSS `border-style`).
///
/// `Dashed`: Dashes three times as long as the border is wide.
///
/// `Dotted`: Round dots as wide as the border.
#[doc(inline)]
pub use kivact_types::BorderStyle;

pub use super::geometry::CornerRadius;

/// How a node's pixels mix with what is beneath (CSS `mix-blend-mode`).
#[doc(inline)]
pub use kivact_types::BlendMode;
