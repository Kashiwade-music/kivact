//! Role: Raster decoding of PNG, JPEG, GIF, WebP and APNG to straight-alpha RGBA8.
//!
//! Raster decoding with the `image` crate: PNG, JPEG, GIF, WebP and APNG
//! to straight-alpha RGBA8. An animation (GIF, WebP, APNG) gives all its
//! frames while they fit a share of the budget, else its first frame.
//!
//! Invariants:
//! - The decoder is chosen by the detected format, never guessed again.
//! - A JPEG's EXIF orientation is applied, and every frame of an animation
//!   gets the same orientation, so the pixels are upright.
//! - Neither edge exceeds `MAX_IMAGE_EDGE`, and the RGBA8 pixels fit the
//!   bytes the caller allows (the store's budget): both are checked from
//!   the header's dimensions before the pixels are allocated. The
//!   decoder's own allocations are limited to `MAX_DECODER_ALLOC`; each
//!   limit fails with its own reason.
//! - An animation's decoded frames, with the frames its decoder holds
//!   besides them, stay within a quarter of the budget. The next frame is
//!   not decoded when it would not fit: the frames decoded so far are kept
//!   (one frame makes it a still image), and the first frame alone when
//!   not even that fits.
//! - A frame delay below 11 ms is 100 ms, as in browsers.

use std::io::Cursor;

use image::codecs::gif::GifDecoder;
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::error::LimitErrorKind;
use image::metadata::{LoopCount, Orientation};
use image::{
    AnimationDecoder, Delay, DynamicImage, Frame, Frames, ImageDecoder, ImageError, ImageFormat,
    ImageReader, Limits,
};
use kivact_types::{Pixels, Repeat};

use crate::format::Format;
use crate::limits::MAX_IMAGE_EDGE;

/// The most the decoder may allocate for one image (its working buffers
/// and the decoded pixels; 512 MiB, the `image` crate's default).
pub(crate) const MAX_DECODER_ALLOC: u64 = 512 * 1024 * 1024;

/// An animation's decoded frames take at most `budget / ANIMATION_SHARE`.
const ANIMATION_SHARE: u64 = 4;

/// Frames a GIF decoder holds besides the frames it returns: the canvas
/// and the raw frame.
const GIF_WORK: u64 = 2;
/// Frames a WebP decoder holds besides the frames it returns: the canvas
/// and the frame's buffer (the bitstream's own tables are not counted).
const WEBP_WORK: u64 = 2;
/// Frames an APNG decoder holds besides the frames it returns: the current
/// and the previous frame, the raw frame and the output copy.
const APNG_WORK: u64 = 4;

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

/// What a raster file decodes to.
pub(crate) enum Decoded {
    /// A still image, or an animation's first frame alone.
    Still(Pixels),
    /// An animation of at least two frames.
    Animated(Animation),
}

/// An animated image: its frames in order, each with its delay in ms, and
/// how often it repeats.
pub(crate) struct Animation {
    /// Each frame's full-canvas pixels and its delay in ms.
    pub(crate) frames: Vec<(Pixels, u32)>,
    /// How often the animation repeats.
    pub(crate) repeat: Repeat,
}

/// Decodes `bytes` of the raster `format` into RGBA pixels. A still image's
/// pixels may take at most `max_bytes`. An animation's frames take at most
/// a quarter of `budget` (see the module invariants).
pub(crate) fn decode(
    bytes: &[u8],
    format: Format,
    max_bytes: u64,
    budget: u64,
) -> Result<Decoded, RasterError> {
    let animated = match format {
        Format::Gif => gif_animation(bytes, budget)?,
        Format::WebP => webp_animation(bytes, budget)?,
        Format::Png => apng_animation(bytes, budget)?,
        Format::Jpeg => None,
        Format::Svg | Format::Svgz => unreachable!("vector formats go to svg::parse"),
    };
    match animated {
        Some(animation) if animation.frames.len() > 1 => Ok(Decoded::Animated(animation)),
        _ => decode_still(bytes, format, max_bytes).map(Decoded::Still),
    }
}

/// The limits every decoder of a still image or an animation runs under.
fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_EDGE);
    limits.max_image_height = Some(MAX_IMAGE_EDGE);
    limits.max_alloc = Some(MAX_DECODER_ALLOC);
    limits
}

/// Decodes the first frame of `bytes` of the raster `format` into RGBA
/// pixels no larger than `max_bytes`.
fn decode_still(bytes: &[u8], format: Format, max_bytes: u64) -> Result<Pixels, RasterError> {
    let format = match format {
        Format::Png => ImageFormat::Png,
        Format::Jpeg => ImageFormat::Jpeg,
        Format::Gif => ImageFormat::Gif,
        Format::WebP => ImageFormat::WebP,
        Format::Svg | Format::Svgz => unreachable!("vector formats go to svg::parse"),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits());
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
    Ok(Pixels {
        width,
        height,
        rgba: pixels.into(),
    })
}

/// A GIF's animation within the frame budget; `None` when it has no
/// frames to give.
fn gif_animation(bytes: &[u8], budget: u64) -> Result<Option<Animation>, RasterError> {
    let mut decoder = GifDecoder::new(Cursor::new(bytes)).map_err(error)?;
    decoder.set_limits(limits()).map_err(error)?;
    let canvas = canvas(&decoder)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let repeat = repeat_of(decoder.loop_count());
    let frames = collect_frames(decoder.into_frames(), orientation, canvas, budget, GIF_WORK)?;
    Ok(animation(frames, repeat))
}

/// A WebP's animation within the frame budget; `None` for a still WebP.
fn webp_animation(bytes: &[u8], budget: u64) -> Result<Option<Animation>, RasterError> {
    let mut decoder = WebPDecoder::new(Cursor::new(bytes)).map_err(error)?;
    if !decoder.has_animation() {
        return Ok(None);
    }
    let canvas = canvas(&decoder)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let repeat = repeat_of(decoder.loop_count());
    let frames = collect_frames(
        decoder.into_frames(),
        orientation,
        canvas,
        budget,
        WEBP_WORK,
    )?;
    Ok(animation(frames, repeat))
}

/// An APNG's animation within the frame budget; `None` for a still PNG.
fn apng_animation(bytes: &[u8], budget: u64) -> Result<Option<Animation>, RasterError> {
    let mut png = PngDecoder::with_limits(Cursor::new(bytes), limits()).map_err(error)?;
    if !png.is_apng().map_err(error)? {
        return Ok(None);
    }
    let canvas = canvas(&png)?;
    let orientation = png.orientation().unwrap_or(Orientation::NoTransforms);
    let decoder = png.apng().map_err(error)?;
    let repeat = repeat_of(decoder.loop_count());
    let frames = collect_frames(
        decoder.into_frames(),
        orientation,
        canvas,
        budget,
        APNG_WORK,
    )?;
    Ok(animation(frames, repeat))
}

/// The canvas size of an animation's decoder, checked against the edge
/// limit before any frame is decoded.
fn canvas<D: ImageDecoder>(decoder: &D) -> Result<(u32, u32), RasterError> {
    let (width, height) = decoder.dimensions();
    if width > MAX_IMAGE_EDGE || height > MAX_IMAGE_EDGE {
        return Err(RasterError::TooLarge);
    }
    if width == 0 || height == 0 {
        return Err(RasterError::Decode("the image has no pixels".to_owned()));
    }
    Ok((width, height))
}

/// Decodes the frames `frames` yields, one at a time, for as long as they
/// fit: before each frame, the frames kept so far, that frame and the
/// `work` frames the decoder holds must fit a quarter of `budget` (each
/// frame is `canvas` in size). Returns the frames kept, with their delays.
fn collect_frames(
    mut frames: Frames<'_>,
    orientation: Orientation,
    canvas: (u32, u32),
    budget: u64,
    work: u64,
) -> Result<Vec<(Pixels, u32)>, RasterError> {
    let frame_bytes = u64::from(canvas.0) * u64::from(canvas.1) * 4;
    let share = budget / ANIMATION_SHARE;
    let mut kept: Vec<(Pixels, u32)> = Vec::new();
    loop {
        if (kept.len() as u64 + 1 + work) * frame_bytes > share {
            tracing::warn!(
                kept = kept.len(),
                "an animation's frames past the decode budget are dropped"
            );
            break;
        }
        let Some(frame) = frames.next() else {
            break;
        };
        let frame: Frame = frame.map_err(error)?;
        let delay = delay_ms(frame.delay());
        let mut image = DynamicImage::ImageRgba8(frame.into_buffer());
        image.apply_orientation(orientation);
        let rgba = image.into_rgba8();
        let (width, height) = rgba.dimensions();
        kept.push((
            Pixels {
                width,
                height,
                rgba: rgba.into_raw().into(),
            },
            delay,
        ));
    }
    Ok(kept)
}

/// A frame delay in ms: below 11 ms it is 100 ms, as in browsers.
fn delay_ms(delay: Delay) -> u32 {
    let (numerator, denominator) = delay.numer_denom_ms();
    let ms = numerator / denominator.max(1);
    if ms < 11 { 100 } else { ms }
}

/// The animation of `frames` repeated `repeat`; `None` when it has none.
fn animation(frames: Vec<(Pixels, u32)>, repeat: Repeat) -> Option<Animation> {
    (!frames.is_empty()).then_some(Animation { frames, repeat })
}

/// The repeat an animation's loop count gives (image's 0 and infinite
/// both mean forever).
fn repeat_of(count: LoopCount) -> Repeat {
    match count {
        LoopCount::Infinite => Repeat::Forever,
        LoopCount::Finite(n) => Repeat::Times(n.get()),
    }
}

fn error(e: ImageError) -> RasterError {
    match e {
        ImageError::Limits(l) => match l.kind() {
            LimitErrorKind::DimensionError => RasterError::TooLarge,
            LimitErrorKind::InsufficientMemory => RasterError::Memory,
            _ => RasterError::Decode(l.to_string()),
        },
        e => RasterError::Decode(e.to_string()),
    }
}
