//! Role: SVG 1.1 export of a recorded drawing, the document of `render_svg`.
//!
//! The drawing is the one `Offscreen` records for its PNG export, with the
//! same theme, fonts and text measurer, so one paint closure and one
//! configuration serve both. The text baseline is measured with the same
//! text engine that the recording uses, so a text lands where a window draws
//! it. Fonts that the document names are not embedded: the viewer needs them
//! installed under the same family names.

mod document;
mod emit;
mod emitter;
mod encode;
mod fill;
mod image;
mod shape;
mod text;

pub(super) use document::write;
