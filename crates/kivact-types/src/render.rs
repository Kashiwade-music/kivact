//! Role: Values the engine and Arvane exchange; the facade re-exports only inline boxes.
//!
//! The host-to-Arvane protocol: drawing, shaping and semantics snapshots a
//! host hands Arvane, and what a window reports back. It uses the
//! vocabulary at the crate root and never the reverse. The one exception a
//! user of the `kivact` facade can name is the pair of inline box types
//! (`InlineBox`, `InlineBoxPlacement`) that `kivact::draw` re-exports for
//! inline paragraphs.

mod composition;
mod effect;
mod paint;
mod scene;
mod semantics;
mod shaped;
mod text;

pub use composition::{
    Composite, CompositeLayer, Composition, Export, ExportFrame, ExportLoss, ExportMode,
    ExportedFrame, ExternalItem, FrameReport, FrameStats, LayerEffects, LayerFilter, LayerItem,
    LayerKey, PixelRect, Plane, RasterGrid, RenderBackdrop, RenderMask, ShaderItem,
};
pub use effect::BoxShadow;
pub use paint::{Gradient, GradientShape, Paint, Pattern, PatternSource};
pub use scene::{CpuFrame, GlyphStroke, Group, RenderCmd, RenderScene, SHADOW_REACH};
pub use semantics::{SemanticsNode, SemanticsTree, SemanticsUpdate};
pub use shaped::{
    Glyph, GlyphRun, GlyphRunSnapshot, InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText,
    ShapedTextSnapshot, TextDecorationRect, TextHighlight, TextLine,
};
pub use text::{RichText, SpanStyle, TextSpan};

/// The most 3D context planes one context may hold (§8.14). Above it the
/// engine and the compositor draw the planes flat, in tree order.
pub const MAX_CONTEXT_PLANES: usize = 256;
