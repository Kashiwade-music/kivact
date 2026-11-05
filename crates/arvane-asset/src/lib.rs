//! Role: Arvane assets: images, SVG and font files.

#![deny(clippy::too_many_lines, clippy::disallowed_types)]
//!
//! Loading, format detection, decoding, SVG rasterization, monochrome
//! tinting, fit geometry, the decoded-image cache and its worker thread,
//! and font file checks live here, apart from drawing (I/O and decoders
//! stay out of the renderer, which only draws decoded RGBA). A host only
//! draws RGBA pixels in a rect, registers font bytes and answers whether a
//! family is installed.
//!
//! # API
//!
//! - [`AssetStore`] loads [`Asset`]s for one host instance, on one worker
//!   thread ([`AssetStore::new`]) or synchronously ([`AssetStore::inline`],
//!   the test driver's mode). [`AssetStore::image`] answers an
//!   [`ImageState`] (pending, ready with the natural size, failed);
//!   [`AssetStore::pixels`] gives the [`AssetImage`] to draw for a device size
//!   and an optional tint ([`AssetStore::pixels_in`] for the visible part
//!   [`Region`] of a clipped image); [`AssetStore::peek`] reads the cache
//!   without starting work; [`AssetStore::begin_frame`] marks a frame of
//!   the cache's recency; [`AssetStore::poll`] applies finished work;
//!   [`AssetStore::take_failures`] and [`AssetStore::take_evicted`] drain
//!   the failures to report and the pixel ids whose device textures can go.
//!   [`AssetStore::release_caches`] evicts eligible unused entries under memory
//!   pressure without changing its normal budget or invalidating recent work.
//! - [`fit_image`] places an image of a natural size in a box by
//!   [`ImageFit`] (CSS `object-fit`); [`PixelSize::of`] turns a drawn size
//!   and a scale into the device size to ask for.
//! - [`load_font`] reads and checks a font file and names its families;
//!   [`resolve_family`] maps a family name or a generic family
//!   ([`GenericFamily`]) to what a host should use, through the host's own
//!   "is this family installed" answer.
//! - Every failure is an [`AssetError`] whose message names the source (the
//!   absolute path and the current directory for files) and the reason.
//!
//! # Invariants
//!
//! - Formats are detected from the content, never from a file extension:
//!   PNG, JPEG (with its EXIF orientation applied), GIF and WebP (first
//!   frame), SVG and SVGZ. Anything else fails with the detected format
//!   named.
//! - AssetImage are straight-alpha RGBA8, `width * height * 4` bytes. A raster
//!   image's natural size is its pixel size (one pixel per logical px); an
//!   SVG's is its `width`/`height`, else its `viewBox`.
//! - Limits: encoded input up to 64 MiB (SVGZ also once decompressed), SVG
//!   nesting up to 256 elements, decoded edges up to 16,384 px, SVG rasters
//!   up to 8,192 device px per edge and a quarter of the budget (only the
//!   visible part of a clipped SVG is rasterized), retained pixels within
//!   the store's budget (256 MiB by default) except what the current and
//!   the previous frame draw once the host begins frames
//!   ([`AssetStore::begin_frame`]).
//! - Depends on `kivact-types` alone among Arvane crates and on no Kivact
//!   crate; no external type (image, resvg, skrifa) appears in its API.
//! - The store lives on its host's UI thread; only its worker thread runs
//!   loads, decodes and rasterizations.

mod error;
mod fit;
mod font;
mod format;
mod limits;
mod pixels;
mod raster;
mod read;
mod source;
mod store;
mod svg;

pub use error::{AssetError, AssetErrorKind};
pub use fit::{FitRect, ImageFit, Region, fit_image};
pub use font::{
    FamilyChoice, FamilyResolution, FontBytes, FontFile, GenericFamily, Os, load_font,
    resolve_family,
};
pub use kivact_types::Asset;
pub use limits::{MAX_ENCODED_BYTES, MAX_IMAGE_EDGE, MAX_SVG_RASTER_EDGE};
pub use pixels::{AssetImage, PixelSize};
pub use store::image_state::ImageState;
pub use store::{AssetStore, DEFAULT_BUDGET};
