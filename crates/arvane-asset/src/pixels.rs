//! Role: Pixel buffers, device sizes for SVG rasters and the monochrome tint.
//!
//! Pixel buffers: what a host draws, the device size it asks SVG rasters
//! for, and the monochrome tint.
//!
//! Invariants:
//! - Pixels are straight-alpha RGBA8 rows, top to bottom, exactly
//!   `width * height * 4` bytes, both edges at least 1.
//! - A [`Pixels`] id is unique within its store and never reused, so a host
//!   may key device textures by it and free them when the store reports the
//!   id evicted.
//! - A tint keeps only the alpha: every pixel becomes the tint color with
//!   alpha `source alpha × tint alpha`.

use std::fmt;
use std::sync::Arc;

use kivact_types::{Rgba, Size};

use crate::fit::Region;

/// Decoded pixels before the store gives them an id. The pixels are
/// already shared (`Arc`), so a decode or raster finished on the worker
/// reaches the UI thread without a copy.
pub(crate) struct Bitmap {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Arc<[u8]>,
}

impl Bitmap {
    /// Bytes the pixels take.
    pub(crate) fn bytes(&self) -> usize {
        self.rgba.len()
    }
}

/// A ready pixel buffer: straight-alpha RGBA8 pixels, with an id stable for as long as the store keeps it.
/// Cloning shares the buffer.
#[derive(Clone)]
pub struct Pixels {
    id: u64,
    width: u32,
    height: u32,
    rgba: Arc<[u8]>,
}

impl Pixels {
    pub(crate) fn new(id: u64, bitmap: Bitmap) -> Self {
        debug_assert_eq!(
            bitmap.rgba.len(),
            bitmap.width as usize * bitmap.height as usize * 4,
            "invariant: width * height * 4 bytes"
        );
        Pixels {
            id,
            width: bitmap.width,
            height: bitmap.height,
            rgba: bitmap.rgba,
        }
    }

    /// The id, unique within the store (see [`AssetStore::take_evicted`]).
    ///
    /// [`AssetStore::take_evicted`]: crate::AssetStore::take_evicted
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The straight-alpha RGBA rows, `width * height * 4` bytes.
    pub fn rgba(&self) -> &Arc<[u8]> {
        &self.rgba
    }

    /// Bytes the pixels take.
    pub(crate) fn bytes(&self) -> usize {
        self.rgba.len()
    }
}

impl PartialEq for Pixels {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Pixels {}

impl fmt::Debug for Pixels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pixels#{}({}x{})", self.id, self.width, self.height)
    }
}

/// A size in device pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PixelSize {
    /// Width in device pixels.
    pub width: u32,
    /// Height in device pixels.
    pub height: u32,
}

impl PixelSize {
    /// The device size of `size` logical px at `scale` device px per logical
    /// px, each edge rounded to the nearest pixel (zero for an empty or
    /// invalid size).
    pub fn of(size: Size, scale: f32) -> PixelSize {
        let edge = |v: f32| {
            let v = (v * scale).round();
            if v.is_finite() && v >= 1.0 {
                v.min(u32::MAX as f32) as u32
            } else {
                0
            }
        };
        PixelSize {
            width: edge(size.width),
            height: edge(size.height),
        }
    }

    /// Whether either edge is zero.
    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// A tint color quantized to RGBA8, the key of a tinted variant.
pub(crate) type Tint = [u8; 4];

/// `color` as a tint key.
pub(crate) fn tint_key(color: Rgba) -> Tint {
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [q(color.r), q(color.g), q(color.b), q(color.a)]
}

/// `source` painted in `tint` (RGBA): its alpha times the tint's alpha, in
/// the tint's color.
pub(crate) fn tint(width: u32, height: u32, source: &[u8], tint: Tint) -> Bitmap {
    let [r, g, b, a] = tint;
    let mut rgba = Vec::with_capacity(source.len());
    for px in source.as_chunks::<4>().0 {
        let alpha = (u32::from(px[3]) * u32::from(a) + 127) / 255;
        rgba.extend_from_slice(&[r, g, b, alpha as u8]);
    }
    Bitmap {
        width,
        height,
        rgba: rgba.into(),
    }
}

/// A part of a raster of `full` device pixels, in device pixels: what an
/// SVG raster of a visible region covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Crop {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Crop {
    /// The device pixels of `full` that cover `visible`, snapped outward to
    /// whole pixels: `None` when nothing is visible, `Some(None)` for the
    /// whole raster.
    pub(crate) fn of(full: PixelSize, visible: Region) -> Option<Option<Crop>> {
        let (w, h) = (f64::from(full.width), f64::from(full.height));
        // A hundredth of a pixel of rounding is not another pixel.
        const SLACK: f64 = 0.01;
        let edge = |from: f32, len: f32, size: f64| {
            let a = (f64::from(from) * size + SLACK).floor().clamp(0.0, size);
            let b = (f64::from(from + len) * size - SLACK)
                .ceil()
                .clamp(0.0, size);
            (a as u32, b as u32)
        };
        let (x0, x1) = edge(visible.x, visible.width, w);
        let (y0, y1) = edge(visible.y, visible.height, h);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        if (x0, y0, x1, y1) == (0, 0, full.width, full.height) {
            return Some(None);
        }
        Some(Some(Crop {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        }))
    }

    /// The part of the image this crop of `full` covers.
    pub(crate) fn region(crop: Option<Crop>, full: PixelSize) -> Region {
        match crop {
            None => Region::WHOLE,
            Some(c) => {
                let (w, h) = (full.width as f32, full.height as f32);
                Region {
                    x: c.x as f32 / w,
                    y: c.y as f32 / h,
                    width: c.width as f32 / w,
                    height: c.height as f32 / h,
                }
            }
        }
    }
}

/// What to rasterize for an SVG drawn at `device` pixels of which
/// `visible` shows: the full device size and the crop, scaled down
/// uniformly so the raster is at most `max_edge` per edge and `max_bytes`
/// of RGBA8 (it is then drawn scaled up). `None` when nothing is visible.
pub(crate) fn raster_plan(
    device: PixelSize,
    visible: Region,
    max_edge: u32,
    max_bytes: usize,
) -> Option<(PixelSize, Option<Crop>)> {
    if device.is_empty() {
        return None;
    }
    let crop = Crop::of(device, visible)?;
    let (w, h) = crop.map_or((device.width, device.height), |c| (c.width, c.height));
    let by_edge = f64::from(max_edge) / f64::from(w.max(h));
    let by_area = ((max_bytes / 4) as f64 / (f64::from(w) * f64::from(h))).sqrt();
    let k = by_edge.min(by_area);
    if k >= 1.0 {
        return Some((device, crop));
    }
    let scaled = |v: u32| ((f64::from(v) * k).floor() as u32).max(1);
    let full = PixelSize {
        width: scaled(device.width),
        height: scaled(device.height),
    };
    Some((full, Crop::of(full, visible)?))
}
