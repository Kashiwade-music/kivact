//! Role: The frame protocol a Kivact host hands Arvane and what a window reports back.
//!
//! The engine produces these values and the Arvane crates consume them:
//! one frame's composition and scenes, the semantics tree, the CPU frame
//! and the frame report. It is a crate of its own because its one reason
//! to change is the renderer contract; the values it builds on (geometry,
//! colors, shaped text) are `kivact-types`.
#![forbid(unsafe_code)]

mod composition;
mod effect;
mod paint;
mod scene;
mod semantics;
mod text;

pub use composition::{
    CompositeLayer, Composition, Export, ExportLoss, ExportMode, ExternalItem, FrameReport,
    LayerEffects, LayerFilter, LayerItem, LayerKey, MAX_CONTEXT_PLANES, PixelRect, Plane,
    RasterGrid, RenderBackdrop, RenderMask, ShaderItem,
};
pub use effect::BoxShadow;
pub use paint::{GradientShape, Pattern, PatternSource, RasterGradient, RasterPaint};
pub use scene::{CpuFrame, GlyphStroke, Group, RenderCmd, RenderScene, SHADOW_REACH};
pub use semantics::{SemanticsNode, SemanticsTree, SemanticsUpdate};
pub use text::{ResolvedSpan, ResolvedSpanStyle, RichText};
