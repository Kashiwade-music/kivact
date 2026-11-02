//! Role: Flattening of node-local canvas scenes into global draw commands.
//!
//! Flatten node-local canvas scenes into global commands at their node's
//! canvas origin, preserving paint order and alpha (scenes keep their size
//! under a transform).
//!
//! Invalid commands are dropped and counted. A pop that does not close the
//! innermost group the scene opened (none is open, or one of another kind) is
//! invalid; the groups a scene leaves open are closed after it, innermost first.
//! A text run uses its own family and line height, else its node's inherited
//! ones (resolved like a text's), its italic, letter spacing, features and decoration
//! lines (drawn like a text's: `glyphs`); an image fills its rect with the asset
//! layer's pixels for the rect × the frame's scale (nothing while loading
//! or failed; the failure goes to the painting [`Context`], which a window
//! ignores).
//!
//! Invariants:
//! - A transform (kivact's `Layer::transform`, already a matrix for the
//!   canvas's size) is pushed as Arvane's `PushTransform` about the canvas
//!   origin, never flattened here, so it applies alike to text, images,
//!   clips, fills, strokes and shadows; under it images are rasterized for
//!   its zoom too (`raster_zoom`). A layer opens as the node's own do
//!   (`effects`: clip, mask, opacity, blend, filters); its color filter
//!   tints what it holds.
//! - A rect clip (`SceneCmd::PushClip`, kivact's `Layer::clip_rect`) opens
//!   no group: it is Arvane's `PushClip`. A layer's clip shape (kivact's
//!   `Layer::clip(&Shape)`, placed in its area) is the clip of the layer's
//!   group.
//! - Pixels (`SceneCmd::Pixels`: straight-alpha RGBA8) are drawn as they
//!   are, through neither the asset store nor its conversion. A part of an
//!   image (`SceneCmd::ImagePart`) is drawn from its source rect, copying no
//!   pixel, rasterized for the destination's device pixels enlarged by how
//!   much the source rect is (an SVG stays sharp, within the store's cache
//!   limit); both are drawn at their opacity, sampled nearest-neighbour
//!   when pixelated.
//! - An external frame (`SceneCmd::External`) is, with feature
//!   `gpu-interop`, Arvane's `DrawCommand::External` at its rect and
//!   opacity: drawn only by the GPU of the window its image was imported
//!   into (without the feature nothing is drawn and no command counts as
//!   dropped). A layer's
//!   color filter does not reach it (Arvane draws it without a color
//!   matrix); one with a rect or source that is not finite, or an opacity
//!   that is not, is dropped.
//! - A scene passes its neutral asset sources directly to the asset store.
//! - A text run with a size or origin that is not finite, or a negative
//!   size, is dropped; one of size 0 draws nothing.
//! - A paragraph (`SceneCmd::Paragraph`) draws the glyph runs and decoration
//!   lines it was shaped with, in its color, at its origin; one with an
//!   origin that is not finite is dropped.

mod geometry;
mod layers;
mod paint;
mod raster;
mod state;
mod text;

pub use raster::raster_zoom;
