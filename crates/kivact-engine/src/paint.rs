//! Role: Conversion of the model's tree into an Arvane composition in paint order.
//!
//! Convert the model's tree into an Arvane composition (one root layer
//! holding a display list) in paint
//! order: the root's tree, then each layer (the fixed boxes outside the
//! overlays, then each overlay in the order they were attached with the
//! fixed boxes inside it) at its window origin, as one flat paint list.
//! Which nodes are drawn, where, at which opacity and in which layers is
//! the retained, culled paint list's (`list`); each frame draws their
//! current content. One walk over a list takes its parameters as a
//! [`state::PaintPass`] (the raster scale, what it draws for and the root
//! layer's size); nothing in the paint state is swapped for it.
//!
//! Invariants:
//! - What a node looks like is the model's evaluation (state
//!   styles, `Model::effective_paint` and `text_style`), sanitized per
//!   node; it is drawn in its own space at its untransformed layout box
//!   (`visual`), through the transform the paint list pushes around it
//!   and its subtree (`model::transform`: what it is hit and exposed
//!   with too).
//! - A hidden node draws nothing below it, overlays attached below it
//!   included (`Model::shown_overlays`). A node's opacity fades its
//!   colors, or, when it draws more than one thing, composites it and its
//!   subtree as one group (`list`); an overlay starts afresh.
//! - A node draws, in CSS order: its outer shadows, its background layers,
//!   its inset shadows, its border and its focus ring (`border`, `fill`),
//!   all in its group (clip path, group opacity, blending: `effects`),
//!   and every color faded by its opacity ([`Tint`]). Color filters,
//!   layer filters, masks and backdrops need compositor layers and are
//!   not drawn here.
//! - `DrawOver` follows the descendants. A clipping node clips its scenes,
//!   text and descendants to its padding box, rounded by its inner radii
//!   (both axes); overlays escape. The clip is a group clipped to the
//!   rounded padding box when it has corners, else a rect clip.
//! - The steps of a node: its transform pushed; with a clip path, its
//!   focus ring straddling the clip path's outline beneath its group; its
//!   group opened; its box (shadows .. ring, when its own paint shows);
//!   its clip; its draw scene (inside the border); its content (a text, a
//!   text input's lines, an image); its descendants; its draw-over scene;
//!   its scrollbars (track, then thumb); then its clip, group and
//!   transform closed. An unbalanced pop in a scene is ignored and what a
//!   scene leaves open is closed after it, so one scene never unbalances
//!   another's.
//! - The focus ring is drawn whenever the paint has one (the core puts it
//!   in the focus state style).
//! - Content that cannot be shaped, or images still loading or failed, are
//!   left out rather than failing the frame; images carry their alpha to
//!   the renderer.
//! - An `Image` node draws the store's pixels of its current asset:
//!   placed in its box (inside the border)
//!   by its `ImageFit`, clipped to the box when it overflows, rasterized
//!   for the drawn rect × the frame's scale × how much the transforms
//!   around it enlarge it (SVGs; `raster_zoom`), in its text color when
//!   monochrome. A scene image fills its rect.
mod frame_scene;
mod list;
pub(crate) mod state;
mod text;
mod text_input;

mod snapshot;
