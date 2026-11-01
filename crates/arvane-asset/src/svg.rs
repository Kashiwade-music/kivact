//! Role: SVG parsing and rasterization at an exact device size.
//!
//! SVG parsing (usvg) and rasterization (resvg on tiny-skia) at an exact
//! device size, so SVGs stay sharp at any scale.
//!
//! Invariants:
//! - Built without resvg's `text` feature and with an empty font database,
//!   so `<text>` is never drawn, the same in every build. Raster images
//!   inside an SVG are not drawn either (resvg's `raster-images` is off).
//! - External files are never read: `<image href>` paths resolve to
//!   nothing, so an SVG cannot make the process read other files.
//! - The natural size is the SVG's `width`/`height`, else its `viewBox`.
//! - A raster is exactly the requested size: the drawing is scaled to it
//!   (the caller asks for the aspect ratio it draws at).
//! - The rasterizer's premultiplied pixels are converted to straight alpha.
//! - SVGZ is decompressed here through a reader limited to
//!   `MAX_ENCODED_BYTES + 1` bytes ([`decompress`]), so a small file that
//!   inflates without bound fails as too large; usvg only ever gets plain
//!   text.
//! - Text nested deeper than `nesting::MAX_SVG_DEPTH` elements, or with
//!   entities that expand to markup, is refused before usvg sees it
//!   (`nesting`); the jobs that parse and rasterize run on a thread with a
//!   large stack (`store::worker`), since usvg and resvg recurse per level.

mod document;
pub(crate) mod nesting;
mod svgz;

pub(crate) use document::{Svg, parse};
pub(crate) use svgz::{DecompressError, decompress};
