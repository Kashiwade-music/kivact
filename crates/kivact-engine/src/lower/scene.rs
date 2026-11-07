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
//!   its zoom too (`raster_zoom`). A layer opens a group (clip, opacity,
//!   blend) unless it needs a compositor layer (a 3D transform, a filter, a
//!   mask, a Porter–Duff operator, or an item inside it): then it opens a
//!   child layer of the composition (`layers`, `layer_effects`).
//! - A rect clip (`DrawCmd::PushClip`, kivact's `Layer::clip_rect`) opens
//!   no group: it is Arvane's `PushClip`. A layer's clip shape (kivact's
//!   `Layer::clip(&Shape)`, placed in its area) is the clip of the layer's
//!   group.
//! - Pixels (`DrawCmd::Pixels`: straight-alpha RGBA8) are drawn as they
//!   are, through neither the asset store nor its conversion. A part of an
//!   image (`DrawCmd::ImagePart`) is drawn from its source rect, copying no
//!   pixel, rasterized for the destination's device pixels enlarged by how
//!   much the source rect is (an SVG stays sharp, within the store's cache
//!   limit); both are drawn at their opacity, sampled nearest-neighbour
//!   when pixelated.
//! - An external frame (`DrawCmd::External`) is a compositor layer item,
//!   carrying its CPU pixels, which the CPU compositor draws where it has
//!   them. A shader fill is a layer item too (`layer_effects`).
//! - A scene passes its neutral asset sources directly to the asset store.
//! - A text run with a size or origin that is not finite, or a negative
//!   size, is dropped; one of size 0 draws nothing.
//! - A paragraph (`DrawCmd::Paragraph`) draws the glyph runs and decoration
//!   lines it was shaped with, in its color, at its origin; one with an
//!   origin that is not finite is dropped.

mod geometry;
mod layer_effects;
mod layers;
mod paint;
mod raster;
mod state;
mod text;

pub(crate) use layers::scene_layers;
pub(crate) use raster::raster_zoom;
