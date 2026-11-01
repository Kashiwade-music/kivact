//! Role: The store's jobs: loading and decoding assets and rasterizing SVGs.
//!
//! The store's work items and how one runs: loading (read, detect, decode
//! or parse) and SVG rasterization. The same code runs on the worker thread
//! and inline.
//!
//! Invariants:
//! - A job owns everything it reads (the asset, a shared parsed SVG), so it
//!   can run on another thread; its result names the entry generation it
//!   was made for, and the store drops results for entries that are gone.
//! - A decoder or rasterizer panic is a bug: it ends the worker thread and
//!   the store resumes it on the host thread at its next poll.
//! - A file's version is taken before it is read, so a file that changes
//!   while being read is seen as changed by the next check.
//! - A check of a failed file (`Job::Recheck`) reads its metadata here, on
//!   the worker (an unreachable network path can block for seconds), and
//!   loads it again only when its version changed.

use std::sync::Arc;

use kivact_types::Asset;

use crate::error::{AssetError, AssetErrorKind};
use crate::format::{self, Format};
use crate::limits::{MAX_ENCODED_BYTES, MAX_IMAGE_EDGE};
use crate::pixels::{Bitmap, Crop, PixelSize};
use crate::raster::{self, RasterError};
use crate::read::{self, FileVersion};
use crate::svg::{self, DecompressError, Svg};

/// Work for the worker (or inline).
pub(crate) enum Job {
    /// Read and decode (or parse) an asset. A raster image's pixels may
    /// take at most `budget - retained` bytes, `retained` being what its
    /// entry holds besides them (`budget` names the cache in the message).
    Load {
        asset: Asset,
        generation: u64,
        budget: usize,
        retained: usize,
    },
    /// Load a failed file again if its version is no longer `version`.
    Recheck {
        asset: Asset,
        generation: u64,
        version: FileVersion,
        budget: usize,
    },
    /// Rasterize a parsed SVG at a device size (only `crop` of it when
    /// given).
    Raster {
        asset: Asset,
        generation: u64,
        svg: Arc<Svg>,
        size: PixelSize,
        crop: Option<Crop>,
    },
}

/// A finished job.
pub(crate) enum Done {
    Load {
        asset: Asset,
        generation: u64,
        version: FileVersion,
        result: Result<Loaded, AssetError>,
    },
    Raster {
        asset: Asset,
        generation: u64,
        size: PixelSize,
        crop: Option<Crop>,
        bitmap: Bitmap,
    },
    /// A failed file's version did not change.
    Unchanged { asset: Asset, generation: u64 },
}

/// A loaded image.
pub(crate) enum Loaded {
    Raster(Bitmap),
    Vector(Arc<Svg>),
}

/// Runs `job` to completion.
pub(crate) fn run(job: Job) -> Done {
    match job {
        Job::Load {
            asset,
            generation,
            budget,
            retained,
        } => {
            let version = match &asset {
                Asset::Path(p) => read::file_version(p),
                _ => None,
            };
            load_done(asset, generation, version, budget, retained)
        }
        Job::Recheck {
            asset,
            generation,
            version,
            budget,
        } => {
            let now = match &asset {
                Asset::Path(p) => read::file_version(p),
                _ => None,
            };
            if now == version {
                Done::Unchanged { asset, generation }
            } else {
                load_done(asset, generation, now, budget, crate::limits::ENTRY_BYTES)
            }
        }
        Job::Raster {
            asset,
            generation,
            svg,
            size,
            crop,
        } => {
            let bitmap = svg.rasterize(size, crop);
            Done::Raster {
                asset,
                generation,
                size,
                crop,
                bitmap,
            }
        }
    }
}

/// The finished load of `asset` (file `version`).
fn load_done(
    asset: Asset,
    generation: u64,
    version: FileVersion,
    budget: usize,
    retained: usize,
) -> Done {
    let result = load(&asset, budget, retained);
    Done::Load {
        asset,
        generation,
        version,
        result,
    }
}

fn load(asset: &Asset, budget: usize, retained: usize) -> Result<Loaded, AssetError> {
    let bytes = read::read(asset)?;
    if bytes.len() as u64 > MAX_ENCODED_BYTES {
        return Err(read::too_large(asset));
    }
    let format = format::detect(&bytes).map_err(|name| {
        AssetError::new(
            AssetErrorKind::UnsupportedFormat,
            asset,
            format!(
                "unsupported image format ({name}); supported: {}",
                format::SUPPORTED
            ),
        )
    })?;
    let decode_error = |reason: String| {
        AssetError::new(
            AssetErrorKind::Decode,
            asset,
            format!("cannot decode {}: {reason}", format.name()),
        )
    };
    if format.is_vector() {
        let plain;
        let text: &[u8] = if format == Format::Svgz {
            plain = svg::decompress(&bytes, MAX_ENCODED_BYTES).map_err(|e| match e {
                DecompressError::TooLarge => AssetError::new(
                    AssetErrorKind::TooLarge,
                    asset,
                    format!(
                        "the decompressed SVGZ is larger than the {} MiB limit for encoded input",
                        MAX_ENCODED_BYTES / (1024 * 1024)
                    ),
                ),
                DecompressError::Malformed(reason) => decode_error(reason),
            })?;
            &plain
        } else {
            &bytes
        };
        return svg::parse(text)
            .map(|svg| Loaded::Vector(Arc::new(svg)))
            .map_err(decode_error);
    }
    let max_bytes = budget.saturating_sub(retained) as u64;
    match raster::decode(&bytes, format, max_bytes) {
        Ok(bitmap) => Ok(Loaded::Raster(bitmap)),
        Err(RasterError::OverBudget {
            width,
            height,
            bytes,
        }) => Err(over_budget(asset, width, height, bytes, budget)),
        Err(RasterError::Memory) => Err(AssetError::new(
            AssetErrorKind::TooLarge,
            asset,
            format!(
                "decoding the {} image needs more than the decoder's {} MiB allocation limit",
                format.name(),
                raster::MAX_DECODER_ALLOC / (1024 * 1024)
            ),
        )),
        Err(RasterError::TooLarge) => Err(AssetError::new(
            AssetErrorKind::TooLarge,
            asset,
            format!(
                "the {} image is larger than {MAX_IMAGE_EDGE} px on an edge",
                format.name()
            ),
        )),
        Err(RasterError::Decode(reason)) => Err(decode_error(reason)),
    }
}

/// The failure of a decoded image of `width` × `height` px (`bytes` of
/// RGBA8) that does not fit a cache of `budget` bytes.
pub(crate) fn over_budget(
    asset: &Asset,
    width: u32,
    height: u32,
    bytes: u64,
    budget: usize,
) -> AssetError {
    let mib = |b: u64| b.div_ceil(1024 * 1024);
    AssetError::new(
        AssetErrorKind::TooLarge,
        asset,
        format!(
            "the decoded image ({width}x{height} px) needs {} MiB, more than the {} MiB image cache",
            mib(bytes),
            mib(budget as u64)
        ),
    )
}
