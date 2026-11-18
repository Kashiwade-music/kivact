//! Role: The facade's drawing types and the windowless renderer.

#[doc(inline)]
pub use kivact_core::draw::{
    BackgroundClip, BackgroundOrigin, BlendMode, BorderImage, BorderImageRepeat, CanvasGradient,
    Composite, CornerRadius, Draw, FillRule, Filter, FilterGraph, FilterInput, FilterShader,
    Gradient, GradientExtend, ImageOptions, InlineBox, InlineBoxPlacement, InlineParagraph,
    IntrinsicSize, Layer, LineCap, LineJoin, Mask, MaskComposite, MaskLayer, MaskMode, Paint,
    PaintShader, Path, PathContour, Pixels, RadialExtent, RegionId, ShaderFilter, ShaderPaint,
    ShaderValue, Shadow, Shape, Stroke, TextBaseline, TextFormat, TextMetrics, TextParagraph,
    TextRun, Transform, conic_gradient, filter, linear_gradient, radial_gradient, text_run,
};
#[doc(inline)]
pub use kivact_offscreen::Offscreen;
