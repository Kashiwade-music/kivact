//! Role: Fixed-resolution images and SVG documents drawn with the `Draw` vocabulary.
//!
//! The path from a `DrawList` through the engine's `paint::Context` to an
//! Arvane scene is the window's. The context keeps the resources, the
//! raster scale and the first failure; text is measured with a
//! `bridge::TextMeasurer` that needs no node tree. Only the CPU
//! rasterizer draws, and its premultiplied output is turned back into
//! straight alpha. The theme's tokens, family and line height come from
//! `Offscreen::theme`, as a window root's do. `render_svg` writes the same
//! recording as an SVG document (see `svg`).
//!
//! Invariants:
//! - A window skips a resource it cannot use and goes on; an `Offscreen`
//!   returns the first failure of an image or a text it actually uses as a
//!   `UiError` of kind `Asset` located at the call (a font that fails to
//!   register too; there is no error type of its own).
//! - Commands `Draw` drops (pixels whose size disagrees with their buffer,
//!   a box-only shadow value) and those the context counts (non-finite or
//!   negative coordinates, sizes and radii, non-finite arcs, rejected
//!   gradient stops or geometry) are `Unsupported`. They are dropped the
//!   way a canvas drops them, invalid geometry requests no resource, and
//!   nothing is dropped silently (a canvas gradient by coordinates that is
//!   rejected paints nothing and is not counted). A window sends what it
//!   dropped once per layer drawing (`HostEvent::DrawDropped { node,
//!   count }`), which the
//!   client reports to the default sink as `Unsupported`; a layer that
//!   drops again after a clean drawing is reported again.
//! - The SVG document of `render_svg` records the drawing once, with the
//!   same theme, fonts and text measurer as `render`. What SVG 1.1 cannot
//!   express is an `Unsupported` error located at the call, never a silent
//!   omission.
//! - The one source of the size limits is Arvane's
//!   `validate::dimensions`/`Rasterizer::check_size`. A size of 0 or over
//!   the limit is not shrunk: it is a `Message` error (the kind of an
//!   invalid `window_size`) and the drawing closure is not called.
//! - Assets load from an inline store and complete at once. Codecs, the
//!   artwork's time and cancelling between frames are the caller's.
mod renderer;
mod svg;

pub use renderer::Offscreen;
