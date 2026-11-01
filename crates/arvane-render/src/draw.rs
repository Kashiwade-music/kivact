//! Role: Drawing of a scene's commands, shared by the CPU and the GPU renderers.
//!
//! Drawing a scene's commands, the same on the CPU and the GPU: the one
//! place that gives every `DrawCommand` its meaning.
//!
//! vello_cpu's render context and vello_hybrid's scene are two backends of
//! the same sparse-strip pipeline (vello_common flattens paths and makes
//! the strips, glifo hints and emboldens glyphs; only the last stage, the
//! pixels, runs on the CPU or the GPU). [`draw`] records every command
//! through [`Canvas`], so both rasterizers receive the same calls and
//! differ only in that last stage, in how glyphs reach it (the GPU draws
//! them from an atlas, at the same positions, `text.rs`), in how images
//! are sampled (from a pixmap on the CPU, from a texture on the GPU) and
//! in how patterns repeat (`Canvas::fill_pattern`).
//!
//! External frames (`DrawCommand::External`, textures an application's
//! render engine shares) are placed, clipped and faded as images are; only
//! the GPU canvas has them, and only frames ready for the frame being
//! drawn (`Canvas::external_size`): elsewhere nothing is drawn. A
//! backdrop's color matrix does not reach them (it changes image pixels on
//! the CPU, which an external frame never is).
//!
//! What vello_hybrid lacks is drawn in a form both have (`effects.rs`):
//! masks and inset shadows remove alpha with destination-out drawing
//! inside a layer, backdrops draw the commands beneath them again (no
//! pixel is read back), color matrices change colors and image pixels
//! before they are drawn (`derived.rs`).
//!
//! State while drawing: the open groups (clips, transforms, layers) and
//! the current transform. A group opened under a transform that flattens
//! everything, or a fully transparent layer, draws nothing up to its pop.
//!
//! Layers, in CSS order: the content is drawn, filtered (blur, drop
//! shadow), clipped, masked, faded by the group opacity and blended with
//! what is beneath. vello's layers clip before they filter, so a layer
//! with a filter is two: a filter-only layer inside a layer holding the
//! clip, the opacity and the blend (the mask is applied between their
//! pops, to the filtered content). The blend modes are peniko's
//! non-destructive mixes (normal to luminosity) and plus-lighter
//! (additive compositing); destination-out is used only inside, for masks
//! and inset shadows.
//!
//! Glyphs under the identity transform, or a translation and a uniform
//! scale (`Transform::uniform_scale`), are hinted and put on quarter
//! device pixels at their transformed position, and may come from the
//! GPU's glyph atlas; under a rotation, skew or non-uniform scale they are
//! drawn from unhinted outlines at their shaped positions (glifo's atlas
//! does not hold rotated glyphs). See `text.rs`.

mod canvas;
mod effects;
mod painter;

pub(crate) use canvas::{Canvas, GlyphRun};
pub(crate) use painter::{Caches, Frame, draw};
