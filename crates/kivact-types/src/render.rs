//! Role: Values only the engine and Arvane exchange; the facade re-exports none.
//!
//! The host-to-Arvane protocol: drawing, shaping and semantics snapshots a
//! host hands Arvane, and what a window reports back. It uses the
//! vocabulary at the crate root and never the reverse, so a value a user
//! of the `kivact` facade can name is never defined here.

mod color;
mod effect;
mod paint;
mod scene;
mod semantics;
mod shaped;
mod text;

pub use color::ColorMatrix;
pub use effect::{Backdrop, BoxShadow, Layer, LayerFilter, Mask};
pub use paint::{Gradient, GradientShape, Paint, Pattern, PatternSource};
pub use scene::{CpuFrame, RenderCmd, RenderScene, SHADOW_REACH};
pub use semantics::{SemanticsNode, SemanticsProperties, SemanticsTree, SemanticsUpdate};
pub use shaped::{
    Glyph, GlyphRun, InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText,
    TextDecorationRect, TextHighlight, TextLine,
};
pub use text::{RichText, SpanStyle, TextSpan};
