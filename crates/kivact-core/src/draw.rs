//! Role: Drawing tools paint canvases and local element layers.
//! Coordinates are node-local logical px after layout. The paints
//! (`Paint`, the gradients), shapes, strokes, filters, blending and
//! transforms are the painter's vocabulary and live here; element styles
//! take the same values (`style().shadow(..)`, `.transform(..)`). Pushed
//! transforms also map text, images and clips.

#[doc(inline)]
pub use kivact_style::effect::{BlendMode, CornerRadius, Filter, Shadow, Shape};
#[doc(inline)]
pub use kivact_style::paint::{
    CanvasGradient, Gradient, GradientExtend, Paint, RadialExtent, conic_gradient, linear_gradient,
    radial_gradient,
};
#[doc(inline)]
pub use kivact_style::transform::Transform;

#[doc(inline)]
pub use kivact_style::draw::{
    Draw, FillRule, ImageOptions, IntrinsicSize, Layer, LineCap, LineJoin, Path, PathContour,
    Pixels, Point, Rect, RegionId, Size, Stroke, TextBaseline, TextMetrics, TextParagraph,
    TextStyle,
};

mod offscreen;
#[doc(inline)]
pub use offscreen::{Offscreen, RgbaImage};
