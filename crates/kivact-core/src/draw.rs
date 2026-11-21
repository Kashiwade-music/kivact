//! Role: Drawing tools paint canvases and local element layers.
//! Coordinates are node-local logical px after layout. CSS values (paints,
//! gradients, shadows, filters, transforms, shapes and masks) are in
//! `kivact::style`; drawing takes them from there. Pushed transforms also map
//! text, images and clips.

#[doc(inline)]
pub use kivact_style::draw::{
    Composite, Draw, FillRule, ImageOptions, InlineBox, InlineBoxPlacement, InlineParagraph, Layer,
    LineCap, LineJoin, Path, PathContour, Pixels, RegionId, Stroke, TextBaseline, TextFormat,
    TextMetrics, TextParagraph,
};
