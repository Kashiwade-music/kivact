//! Role: Pixels derived from a scene's values and kept from one frame to the next.
//!
//! Pixels the renderer makes from a scene's values, kept from one frame to
//! the next: images through a color matrix, gradient tiles rasterized at a
//! device size, and pattern images resampled to their tile's device size. Both renderers draw them as ordinary images (their
//! own caches key them by these pixels' identity).
//!
//! Invariants (the frame cache rules of `images.rs`):
//! - A filtered or resampled image is keyed by its source's pixel identity
//!   (the `Arc` address, kept alive by the entry), dimensions and the
//!   matrix or new size; a tile by the gradient's values, the tile's size
//!   and its pixel size.
//! - An entry not used in a frame is dropped when that frame ends; at most
//!   `MAX_BYTES` of pixels are kept (beyond that a result is made for its
//!   frame and not kept).
//! - Thread: owned by its renderer, used on the renderer's thread.

use std::collections::HashMap;
use std::sync::Arc;

use kivact_types::{ColorMatrix, Extend, Gradient, GradientShape, ImageData, Rect};
use vello_cpu::{
    Image, ImageSource, PaintType, Pixmap, RenderContext, RenderSettings, Resources,
    kurbo::{Affine, Rect as RasterRect},
    peniko::{self, Blob, ImageAlphaType, ImageFormat, ImageQuality, ImageSampler},
};

use crate::gradient;

/// The derived pixels kept at most (64 MiB).
const MAX_BYTES: usize = 64 * 1024 * 1024;

/// A filtered image: source address and size, and the matrix's bits.
type FilterKey = (usize, u32, u32, [u32; 20]);

/// A gradient tile: the gradient's and the tile's values as bits, and the
/// pixel size.
type TileKey = (Vec<u32>, u16, u16);

/// A resampled image: source address and size, and the new size.
type ResampleKey = (usize, u32, u32, u16, u16);

struct Entry {
    /// Keeps a filtered image's source address from being reused.
    _source: Option<Arc<[u8]>>,
    image: ImageData,
    frame: u64,
}

/// Derived pixels by what they are made from (see the module invariants).
#[derive(Default)]
pub(crate) struct Derived {
    filtered: HashMap<FilterKey, Entry>,
    tiles: HashMap<TileKey, Entry>,
    resampled: HashMap<ResampleKey, Entry>,
    /// The rasterizer of gradient tiles, made on first use.
    context: Option<(RenderContext, Resources)>,
    frame: u64,
    bytes: usize,
}

impl Derived {
    /// Starts a frame.
    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;
    }

    /// Ends the frame: drops what it did not use.
    pub(crate) fn end_frame(&mut self) {
        let frame = self.frame;
        let mut freed = 0;
        let mut keep = |e: &mut Entry| {
            let keep = e.frame == frame;
            if !keep {
                freed += e.image.rgba.len();
            }
            keep
        };
        self.filtered.retain(|_, e| keep(e));
        self.tiles.retain(|_, e| keep(e));
        self.resampled.retain(|_, e| keep(e));
        self.bytes -= freed;
    }

    /// `image`'s pixels through `matrix` (straight alpha).
    pub(crate) fn filtered(&mut self, image: &ImageData, matrix: &ColorMatrix) -> ImageData {
        let key = (
            Arc::as_ptr(&image.rgba).cast::<u8>() as usize,
            image.width,
            image.height,
            matrix.0.map(f32::to_bits),
        );
        if let Some(entry) = self.filtered.get_mut(&key) {
            entry.frame = self.frame;
            return entry.image.clone();
        }
        let mut rgba = Vec::with_capacity(image.rgba.len());
        for px in image.rgba.as_chunks::<4>().0 {
            let c = px.map(|v| f32::from(v) / 255.0);
            rgba.extend(matrix.apply_array(c).map(|v| (v * 255.0).round() as u8));
        }
        let out = ImageData {
            width: image.width,
            height: image.height,
            rgba: rgba.into(),
        };
        let source = Some(Arc::clone(&image.rgba));
        Self::keep(
            &mut self.filtered,
            &mut self.bytes,
            self.frame,
            key,
            source,
            &out,
        );
        out
    }

    /// `gradient` over a tile of `tile`'s size, rasterized at `size` pixels
    /// (straight alpha).
    pub(crate) fn gradient_tile(
        &mut self,
        gradient: &Gradient,
        tile: Rect,
        (width, height): (u16, u16),
    ) -> ImageData {
        let key = (tile_bits(gradient, tile), width, height);
        if let Some(entry) = self.tiles.get_mut(&key) {
            entry.frame = self.frame;
            return entry.image.clone();
        }
        let out = self.raster((width, height), |context| {
            context.set_transform(Affine::scale_non_uniform(
                f64::from(width) / f64::from(tile.width),
                f64::from(height) / f64::from(tile.height),
            ));
            let (brush, transform) = gradient::gradient(gradient, None);
            context.set_paint(brush);
            if let Some(transform) = transform {
                context.set_paint_transform(transform);
            }
            context.fill_rect(&RasterRect::new(
                0.0,
                0.0,
                f64::from(tile.width),
                f64::from(tile.height),
            ));
        });
        Self::keep(
            &mut self.tiles,
            &mut self.bytes,
            self.frame,
            key,
            None,
            &out,
        );
        out
    }

    /// `image` resampled (bilinear, edges clamped) to `size` pixels
    /// (straight alpha): a pattern tile at its device size, so both
    /// rasterizers repeat it pixel for pixel.
    pub(crate) fn resampled(
        &mut self,
        image: &ImageData,
        (width, height): (u16, u16),
    ) -> ImageData {
        if (image.width, image.height) == (u32::from(width), u32::from(height)) {
            return image.clone();
        }
        let key = (
            Arc::as_ptr(&image.rgba).cast::<u8>() as usize,
            image.width,
            image.height,
            width,
            height,
        );
        if let Some(entry) = self.resampled.get_mut(&key) {
            entry.frame = self.frame;
            return entry.image.clone();
        }
        let source = ImageSource::from_peniko_image_data(&peniko::ImageData {
            data: Blob::new(Arc::new(Arc::clone(&image.rgba))),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width: image.width,
            height: image.height,
        });
        let out = self.raster((width, height), |context| {
            context.set_paint(PaintType::Image(Image {
                image: source,
                sampler: ImageSampler {
                    x_extend: peniko::Extend::Pad,
                    y_extend: peniko::Extend::Pad,
                    quality: ImageQuality::Medium,
                    alpha: 1.0,
                },
            }));
            context.set_paint_transform(Affine::scale_non_uniform(
                f64::from(width) / f64::from(image.width),
                f64::from(height) / f64::from(image.height),
            ));
            context.fill_rect(&RasterRect::new(
                0.0,
                0.0,
                f64::from(width),
                f64::from(height),
            ));
        });
        let source = Some(Arc::clone(&image.rgba));
        Self::keep(
            &mut self.resampled,
            &mut self.bytes,
            self.frame,
            key,
            source,
            &out,
        );
        out
    }

    /// The straight-alpha pixels of what `draw` draws into a `width` ×
    /// `height` frame.
    fn raster(
        &mut self,
        (width, height): (u16, u16),
        draw: impl FnOnce(&mut RenderContext),
    ) -> ImageData {
        let (context, resources) = self.context.get_or_insert_with(|| {
            let settings = RenderSettings {
                level: vello_cpu::Level::new(),
                num_threads: 0,
            };
            (
                RenderContext::new_with(width, height, settings),
                Resources::new(),
            )
        });
        context.reset_and_resize(width, height);
        draw(context);
        context.reset_paint_transform();
        context.flush();
        let mut pixmap = Pixmap::new(width, height);
        context.render(&mut pixmap, resources);
        let mut rgba = Vec::with_capacity(usize::from(width) * usize::from(height) * 4);
        for p in pixmap.data() {
            let unpremultiply = |c: u8| {
                if p.a == 0 {
                    0
                } else {
                    ((u32::from(c) * 255 + u32::from(p.a) / 2) / u32::from(p.a)).min(255) as u8
                }
            };
            rgba.extend([
                unpremultiply(p.r),
                unpremultiply(p.g),
                unpremultiply(p.b),
                p.a,
            ]);
        }
        ImageData {
            width: u32::from(width),
            height: u32::from(height),
            rgba: rgba.into(),
        }
    }

    fn keep<K: std::hash::Hash + Eq>(
        map: &mut HashMap<K, Entry>,
        bytes: &mut usize,
        frame: u64,
        key: K,
        source: Option<Arc<[u8]>>,
        image: &ImageData,
    ) {
        let size = image.rgba.len();
        if *bytes + size <= MAX_BYTES {
            *bytes += size;
            map.insert(
                key,
                Entry {
                    _source: source,
                    image: image.clone(),
                    frame,
                },
            );
        }
    }
}

/// A gradient tile's values as bits.
fn tile_bits(g: &Gradient, tile: Rect) -> Vec<u32> {
    let mut bits = Vec::with_capacity(12 + g.stops.len() * 5);
    bits.extend([tile.width, tile.height].map(f32::to_bits));
    bits.push(match g.extend {
        Extend::Pad => 0,
        Extend::Repeat => 1,
        Extend::Reflect => 2,
    });
    match g.shape {
        GradientShape::Linear { start, end } => {
            bits.push(0);
            bits.extend([start.x, start.y, end.x, end.y].map(f32::to_bits));
        }
        GradientShape::Radial {
            center,
            radius_x,
            radius_y,
        } => {
            bits.push(1);
            bits.extend([center.x, center.y, radius_x, radius_y].map(f32::to_bits));
        }
        GradientShape::RadialTwoCircle {
            start_center,
            start_radius,
            end_center,
            end_radius,
        } => {
            bits.push(3);
            bits.extend(
                [
                    start_center.x,
                    start_center.y,
                    start_radius,
                    end_center.x,
                    end_center.y,
                    end_radius,
                ]
                .map(f32::to_bits),
            );
        }
        GradientShape::Conic {
            center,
            start_deg,
            end_deg,
        } => {
            bits.push(2);
            bits.extend([center.x, center.y, start_deg, end_deg].map(f32::to_bits));
        }
    }
    for (offset, c) in &g.stops {
        bits.extend([*offset, c.r, c.g, c.b, c.a].map(f32::to_bits));
    }
    bits
}
