//! Role: Raster decoding of PNG, JPEG, GIF and WebP to straight-alpha RGBA8.
//!
//! Raster decoding with the `image` crate: PNG, JPEG, GIF and WebP to
//! straight-alpha RGBA8.
//!
//! Invariants:
//! - The decoder is chosen by the detected format, never guessed again.
//! - A JPEG's EXIF orientation is applied, so the pixels are upright.
//! - GIF and WebP give their first frame (animation is out of scope).
//! - Neither edge exceeds `MAX_IMAGE_EDGE`, and the RGBA8 pixels fit the
//!   bytes the caller allows (the store's budget): both are checked from
//!   the header's dimensions before the pixels are allocated. The
//!   decoder's own allocations are limited to `MAX_DECODER_ALLOC`; each
//!   limit fails with its own reason.

use std::io::Cursor;

use image::error::LimitErrorKind;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};

use crate::format::Format;
use crate::limits::MAX_IMAGE_EDGE;
use crate::pixels::Bitmap;

/// The most the decoder may allocate for one image (its working buffers
/// and the decoded pixels; 512 MiB, the `image` crate's default).
pub(crate) const MAX_DECODER_ALLOC: u64 = 512 * 1024 * 1024;

/// Why a raster image did not decode.
pub(crate) enum RasterError {
    /// An edge exceeds `MAX_IMAGE_EDGE`.
    TooLarge,
    /// The decoded RGBA8 pixels (`width` × `height`, `bytes`) would exceed
    /// the bytes allowed.
    OverBudget { width: u32, height: u32, bytes: u64 },
    /// The decoder would allocate more than `MAX_DECODER_ALLOC`.
    Memory,
    /// The decoder failed.
    Decode(String),
}

/// Decodes `bytes` of the raster `format`, whose RGBA8 pixels may take at
/// most `max_bytes`, into RGBA pixels.
pub(crate) fn decode(bytes: &[u8], format: Format, max_bytes: u64) -> Result<Bitmap, RasterError> {
    let format = match format {
        Format::Png => ImageFormat::Png,
        Format::Jpeg => ImageFormat::Jpeg,
        Format::Gif => ImageFormat::Gif,
        Format::WebP => ImageFormat::WebP,
        Format::Svg | Format::Svgz => unreachable!("vector formats go to svg::parse"),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_EDGE);
    limits.max_image_height = Some(MAX_IMAGE_EDGE);
    limits.max_alloc = Some(MAX_DECODER_ALLOC);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(error)?;
    let (width, height) = decoder.dimensions();
    if width > MAX_IMAGE_EDGE || height > MAX_IMAGE_EDGE {
        return Err(RasterError::TooLarge);
    }
    let rgba_bytes = u64::from(width) * u64::from(height) * 4;
    if rgba_bytes > max_bytes {
        return Err(RasterError::OverBudget {
            width,
            height,
            bytes: rgba_bytes,
        });
    }
    if rgba_bytes > MAX_DECODER_ALLOC {
        return Err(RasterError::Memory);
    }
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder).map_err(error)?;
    image.apply_orientation(orientation);
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 {
        return Err(RasterError::Decode("the image has no pixels".to_owned()));
    }
    let pixels = rgba.into_raw();
    Ok(Bitmap {
        width,
        height,
        rgba: pixels.into(),
    })
}

fn error(e: image::ImageError) -> RasterError {
    match e {
        image::ImageError::Limits(l) => match l.kind() {
            LimitErrorKind::DimensionError => RasterError::TooLarge,
            LimitErrorKind::InsufficientMemory => RasterError::Memory,
            _ => RasterError::Decode(l.to_string()),
        },
        e => RasterError::Decode(e.to_string()),
    }
}
