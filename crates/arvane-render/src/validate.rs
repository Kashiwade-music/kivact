//! Role: Validate scene data before rasterization.
//! No invalid float, out-of-range
//! value or unmatched group enters the rasterizers.
//!
//! A scene is checked whole before anything is drawn ([`scene`]), so an
//! invalid one draws nothing and leaves the renderer usable: every number
//! finite, color components and gradient stop offsets in `0..=1` (stops
//! not decreasing), sizes and widths not negative, groups (clips,
//! transforms, layers) properly nested, each pop closing the innermost
//! group of its kind, at most [`MAX_SCENE_DEPTH`](crate::MAX_SCENE_DEPTH) open at once, blurs of at most
//! `MAX_BLUR` standard deviations, at most `MAX_DASHES` dash lengths, glyph
//! runs whose font skrifa reads and whose glyph ids it has. Images are
//! decoded straight-alpha RGBA8 whose bytes match their dimensions, edges
//! within 16,384 px ([`image`](scene::image)); this crate decodes no encoded image.
//!
//! The frame budget (each side 16,384 physical px, 16,777,216 pixels: 64
//! MiB of RGBA8) is decided only here: [`dimensions`] says whether a frame
//! is drawn, and [`raster_plan`] picks, from a window's physical size and
//! scale, the logical extent (never rasterizing beyond the window) and the
//! raster scale, reduced for a window beyond the budget (its frame is then
//! enlarged to the window when it is presented).

mod fonts;
mod frame;
mod scene;

pub(crate) use fonts::Fonts;
pub use frame::{dimensions, raster_plan};
#[cfg(feature = "gpu")]
pub(crate) use scene::SceneInfo;
pub(crate) use scene::scene;
