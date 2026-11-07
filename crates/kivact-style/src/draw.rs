//! Role: The `Draw` painter: shapes, paths, text, images, layers and canvas regions.
//! Coordinates are node-local logical px after layout.
//! A layer's transform maps every primitive, including text, images and
//! clips, through the renderer. Regions retain their shapes and transforms for hits.
//! A style's `paint(..)` callback is kept here as a `LocalPaint` (see `CompiledStyle::paint_retained`).
//!
//! `canvas`, `draw` and `draw_over` build the wire `DrawList`. A group opens
//! only with `d.layer(Layer, closure)` and closes at the end of the
//! closure; there is no public pop. A [`Layer`] holds a transform, a clip
//! (a `Shape`, a [`Path`] or a rectangle) and opacity, blend, filter, mask
//! and shadow, pushed from the outside in: transform, rectangle clip, then
//! `PushLayer`. The transform goes to the renderer as `PushTransform`
//! (style does not flatten it) and applies to text, images, clips, fills,
//! strokes and shadows alike. `line` and `stroke_rect` take a `Stroke` and
//! a `Paint` (a number becomes a solid stroke of that width): a plain solid
//! line keeps its direct command, while caps, joins, dashes and gradients
//! go the way of `stroke_path`. Fills and strokes of paths and shapes share
//! `Paint`, `Shape` and `Stroke` (cap, join, miter, dash, offset); color
//! filters (tint) and layer filters and blending share `paint::effects`
//! with the element style, so a canvas adds no meaning of its own. Rare
//! command values are boxed so an ordinary rectangle scene stays small.
//!
//! A text's `TextFormat.align` is the shared `TextAlign` (Start, Center,
//! End) and places the text horizontally against its anchor (not the
//! style's child alignment); with [`TextBaseline`] (Top, Middle,
//! Alphabetic, Bottom) it places the text by the extent and baseline of the
//! same shaping as `measure_text`. The default Top/Start measures nothing
//! extra.
//!
//! Regions:
//! - A canvas reads the size signal of its node, so its first drawing is
//!   at size 0 and it draws again after the first layout report.
//! - `region`, `region_shape` and `region_path` make an accessibility leaf
//!   and a hit area directly under the node, for a region the node's setup
//!   declared with `Node::region(id, part)` (or `Node::regions`): its role,
//!   name, states (`checked`, `selected`, `expanded`, `indeterminate`, ...)
//!   and input come from that part. A switch track, a selectable item or a
//!   disclosure on a canvas is thus its own native accessibility leaf, and a
//!   state change is published by the next drawing; `kivact::test`'s
//!   `Element` state queries read the region's published state. This is
//!   meaning only: the pointer and the state updates are the component's
//!   handlers.
//! - A region keeps the transform at registration and its original shape:
//!   a hit maps the point back and tests the same outline the renderer
//!   draws. Its accessibility bounds are the bounding box of the
//!   transformed rectangle. Draw and layer clips do not cut a region's hit
//!   area; the node's own clip and transform apply.
//! - `PointerEvent::region` and `A11yEvent::region` carry the author's
//!   `RegionId`; the node itself is `None`.
//! - An undeclared id, or geometry repeated in one layer, is reported and
//!   dropped. Pixels whose size disagrees with their buffer and a canvas
//!   shadow's spread or inset are `Unsupported`, reported once per layer,
//!   located at the node; the engine's scene culling includes the reach of
//!   transforms, strokes and filters.

mod border_image;
mod drawable;
pub mod filter;
mod image;
mod layer;
mod local_paint;
mod paragraph;
pub(crate) mod recorder;
mod regions;
mod shapes;
mod text_run;
pub use super::geometry::{IntrinsicSize, Point, Rect, Size};
pub use super::path::{FillRule, Path, PathContour};
pub use image::GpuFrame;
pub use image::{ImageOptions, Pixels};
pub use kivact_types::{InlineBox, InlineBoxPlacement};
pub use layer::Layer;
pub use paragraph::{InlineParagraph, TextParagraph};

pub use kivact_types::RegionId;

/// Ends of open contours and dashes.
#[doc(inline)]
pub use kivact_types::LineCap;
/// How adjacent segments meet.
#[doc(inline)]
pub use kivact_types::LineJoin;
/// A line's width, caps, joins and repeating dash pattern, in logical px.
#[doc(inline)]
pub use kivact_types::Stroke;

/// The extent of measured text: [`Draw::measure_text`] and
/// `Constraints::measure_text` of `kivact::custom`,
/// what `Draw::text` draws (one line, unwrapped) in logical px.
/// `width` is the line advance, `height` is its height, and `baseline`
/// is measured from its top (the `pos` passed to `Draw::text`).
#[doc(inline)]
pub use kivact_types::TextMetrics;

pub(crate) use local_paint::LocalPaint;
pub use recorder::{Draw, Recording, RegionSemantics};
pub use text_run::{TextBaseline, TextFormat, text_run};

pub use kivact_types::TextRun;

pub use crate::effect::{FilterGraph, FilterInput};
pub use crate::paint::{
    BackgroundClip, BackgroundOrigin, BorderImage, BorderImageRepeat, BorderImageSource, Mask,
    MaskComposite, MaskLayer, MaskMode,
};
pub use crate::shader::{FilterShader, PaintShader, ShaderFilter, ShaderPaint, ShaderValue};
pub use drawable::DrawableSource;
pub use kivact_types::render::Composite;
