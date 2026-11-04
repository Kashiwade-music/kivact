//! Role: Values only the engine and Arvane exchange; the facade re-exports none.
//!
//! The host-to-Arvane protocol: drawing, shaping and semantics snapshots a
//! host hands Arvane, and what a window reports back. It uses the
//! vocabulary at the crate root and never the reverse, so a value a user
//! of the `kivact` facade can name is never defined here.

mod color;
mod effect;
mod image;
mod paint;
mod scene;
mod semantics;
pub(crate) mod shape;
mod shaped;
mod text;
mod transform;
mod window;

pub use color::ColorMatrix;
pub use effect::{Backdrop, BoxShadow, Layer, LayerFilter, Mask};
pub use image::ImageData;
pub use paint::{Gradient, GradientShape, Paint, Pattern, PatternSource};
pub use scene::{CpuFrame, DrawCommand, SHADOW_REACH, Scene};
pub use semantics::{SemanticsNode, SemanticsTree, SemanticsUpdate};
pub use shape::{PathCmd, Radius, RoundedRect, Shape};
pub use shaped::{
    Glyph, GlyphRun, InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText,
    TextDecorationRect, TextHighlight, TextLine,
};
pub use text::{FontFeature, RichText, SpanStyle, TextDecoration, TextSpan, TextStyle, TextWrap};
pub use transform::Transform;
pub use window::{Environment, MouseSettings, ParentWindow, PointerInput};
