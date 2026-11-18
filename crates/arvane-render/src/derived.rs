//! Role: Pixels derived from a scene's values and kept from one frame to the next.
//!
//! Pixels the renderer makes from a scene's values, kept from one frame to
//! the next: gradient tiles rasterized at a device size, and pattern images
//! resampled to their tile's device size. Both rasterizers draw them as
//! ordinary images (their own caches key them by these pixels' identity).
//!
//! Invariants (the frame cache's rules, `frame_cache.rs`):
//! - A resampled image is keyed by its source's pixel identity (the `Arc`
//!   address, pinned by the entry), dimensions and new size; a tile by the
//!   gradient's values, the tile's size and its pixel size.
//! - Both kinds share one budget of `MAX_BYTES`. An entry not used in a
//!   frame is dropped when that frame ends; a result beyond the budget is
//!   made for its frame and not kept.
//! - Thread: owned by its renderer, used on the renderer's thread.

use std::sync::Arc;

use arvane_protocol::{GradientShape, RasterGradient};
use kivact_types::pixel::Pixels;
use kivact_types::{Extend, Rect};
use vello_cpu::kurbo::{Affine, Rect as RasterRect};
use vello_cpu::peniko::{self, Blob, ImageAlphaType, ImageFormat, ImageQuality, ImageSampler};
use vello_cpu::{Image, ImageSource, PaintType, Pixmap, RenderContext, RenderSettings, Resources};

use crate::frame_cache::{FrameCache, FrameScope, Pinned};
use crate::{gradient, images};

/// The derived pixels kept at most (64 MiB, both kinds together).
const MAX_BYTES: usize = 64 * 1024 * 1024;

/// A gradient tile: the gradient's and the tile's values as bits, and the
/// pixel size.
type TileKey = (Vec<u64>, u16, u16);

/// A resampled image: the source's identity and size, and the new size.
type ResampleKey = (images::ImageKey, u16, u16);

/// What a derived pixel is made from. Both kinds share one cache.
#[derive(Hash, PartialEq, Eq)]
enum DerivedKey {
    Tile(TileKey),
    Resample(ResampleKey),
}

/// Derived pixels by what they are made from (see the module invariants).
pub(crate) struct Derived {
    cache: FrameCache<DerivedKey, Pinned<Pixels>>,
    /// The rasterizer of gradient tiles and resampled images, made on first use.
    context: Option<(RenderContext, Resources)>,
}

impl Default for Derived {
    fn default() -> Self {
        Self {
            cache: FrameCache::new(MAX_BYTES),
            context: None,
        }
    }
}

impl Derived {
    /// Draws one frame: `body` uses the derived pixels through the scope it is given.
    pub(crate) fn frame<R>(&mut self, body: impl FnOnce(DerivedScope<'_>) -> R) -> R {
        let Derived { cache, context } = self;
        cache.frame(|scope| body(DerivedScope { scope, context }))
    }
}

/// The derived pixels of the frame being drawn.
pub(crate) struct DerivedScope<'a> {
    scope: FrameScope<'a, DerivedKey, Pinned<Pixels>>,
    context: &'a mut Option<(RenderContext, Resources)>,
}

impl DerivedScope<'_> {
    /// `gradient` over a tile of `tile`'s size, rasterized at `size` pixels
    /// (straight alpha).
    pub(crate) fn gradient_tile(
        &mut self,
        gradient: &RasterGradient,
        tile: Rect,
        (width, height): (u16, u16),
    ) -> Pixels {
        let key = DerivedKey::Tile((tile_bits(gradient, tile), width, height));
        self.scope
            .get_or_insert_with(key, || {
                let image = raster(&mut *self.context, (width, height), |context| {
                    context.set_transform(Affine::scale_non_uniform(
                        f64::from(width) / tile.width,
                        f64::from(height) / tile.height,
                    ));
                    let (brush, transform) = gradient::gradient(gradient);
                    context.set_paint(brush);
                    if let Some(transform) = transform {
                        context.set_paint_transform(transform);
                    }
                    context.fill_rect(&RasterRect::new(0.0, 0.0, tile.width, tile.height));
                });
                let cost = image.rgba.len();
                (Pinned::unpinned(image), cost)
            })
            .value
    }

    /// `image` resampled (bilinear, edges clamped) to `size` pixels
    /// (straight alpha): a pattern tile at its device size, so both
    /// rasterizers repeat it pixel for pixel.
    pub(crate) fn resampled(&mut self, image: &Pixels, (width, height): (u16, u16)) -> Pixels {
        if (image.width, image.height) == (u32::from(width), u32::from(height)) {
            return image.clone();
        }
        let key = DerivedKey::Resample((images::image_key(image), width, height));
        self.scope
            .get_or_insert_with(key, || {
                let source = ImageSource::from_peniko_image_data(&peniko::ImageData {
                    data: Blob::new(Arc::new(Arc::clone(image.rgba.as_shared()))),
                    format: ImageFormat::Rgba8,
                    alpha_type: ImageAlphaType::Alpha,
                    width: image.width,
                    height: image.height,
                });
                let out = raster(&mut *self.context, (width, height), |context| {
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
                let cost = out.rgba.len();
                (Pinned::new(out, Arc::clone(image.rgba.as_shared())), cost)
            })
            .value
    }
}

/// The straight-alpha pixels of what `draw` draws into a `width` ×
/// `height` frame, rasterized with `context` (made on first use).
fn raster(
    context: &mut Option<(RenderContext, Resources)>,
    (width, height): (u16, u16),
    draw: impl FnOnce(&mut RenderContext),
) -> Pixels {
    let (context, resources) = context.get_or_insert_with(|| {
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
    for pixel in pixmap.data() {
        rgba.extend([pixel.r, pixel.g, pixel.b, pixel.a]);
    }
    kivact_types::color::unpremultiply(&mut rgba);
    Pixels {
        width: u32::from(width),
        height: u32::from(height),
        rgba: rgba.into(),
    }
}

/// A gradient tile's values as bits.
fn tile_bits(g: &RasterGradient, tile: Rect) -> Vec<u64> {
    let mut bits = Vec::with_capacity(12 + g.stops.len() * 5);
    bits.extend([tile.width, tile.height].map(f64::to_bits));
    bits.push(match g.extend {
        Extend::Pad => 0,
        Extend::Repeat => 1,
        Extend::Reflect => 2,
    });
    match g.shape {
        GradientShape::Linear { start, end } => {
            bits.push(0);
            bits.extend([start.x, start.y, end.x, end.y].map(f64::to_bits));
        }
        GradientShape::Radial {
            center,
            radius_x,
            radius_y,
        } => {
            bits.push(1);
            bits.extend([center.x, center.y, radius_x, radius_y].map(f64::to_bits));
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
                .map(f64::to_bits),
            );
        }
        GradientShape::Conic {
            center,
            start_deg,
            end_deg,
        } => {
            bits.push(2);
            bits.extend([center.x, center.y, start_deg, end_deg].map(f64::to_bits));
        }
    }
    for (offset, c) in &g.stops {
        bits.extend([*offset, c.r, c.g, c.b, c.a].map(f64::to_bits));
    }
    bits
}
