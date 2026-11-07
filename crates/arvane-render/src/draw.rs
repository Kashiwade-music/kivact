//! Role: Drawing of a scene's commands, shared by the CPU and the GPU rasterizers.
//!
//! Drawing a scene's commands, the same on the CPU and the GPU: the one
//! place that gives every `RenderCmd` its meaning.
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
//! The scene is mapped by a root transform (`Frame::origin` and
//! `Frame::scale`) before its own transforms, so one raster can show any
//! part of a scene. Groups are the scene's own: a [`RenderCmd::PushGroup`]
//! is clipped, faded and blended as one layer, and its blur is a filter
//! layer inside it (vello's layers clip before they filter, so the blur
//! is applied first, as CSS orders filter, clip-path and opacity).
//!
//! State while drawing: the open groups (clips, transforms, groups) and
//! the current transform. A group opened under a transform that flattens
//! everything, or a fully transparent group, draws nothing up to its pop.
//!
//! Glyphs under the identity transform, or a translation and a uniform
//! scale (`Transform::uniform_scale`), are hinted and put on quarter
//! device pixels at their transformed position, and may come from the
//! GPU's glyph atlas; under a rotation, skew or non-uniform scale they are
//! drawn from unhinted outlines at their shaped positions (glifo's atlas
//! does not hold rotated glyphs). See `text.rs`.

mod canvas;
mod painter;
mod shadows;

pub(crate) use canvas::{Canvas, GlyphRun};
pub(crate) use painter::{Caches, Frame, draw};
