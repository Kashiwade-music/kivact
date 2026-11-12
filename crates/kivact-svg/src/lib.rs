//! Role: Writes a recorded Kivact drawing as an SVG 1.1 document.
//!
//! The input is a recorded `DrawList` and its logical size; the output is
//! the document text. The writer asks the renderer that recorded the drawing
//! for three things through [`Resources`]: the fallback font family, the
//! baseline of a text run as the text engine measures it, and the pixels of
//! one image command drawn at a size. Nothing here looks into the engine, so
//! the writer is tested without a window.
//!
//! This is a crate because its one reason to change is the mapping of the
//! draw commands to SVG 1.1, and because the PNG encoding and the SVG
//! knowledge stay in one leaf. `kivact-core` (`Offscreen::render_svg`) is its
//! only user.

mod document;
mod emit;
mod emitter;
mod encode;
mod error;
mod fill;
mod image;
mod resources;
mod shape;
mod text;

pub use document::write;
pub use error::SvgError;
pub use resources::Resources;
