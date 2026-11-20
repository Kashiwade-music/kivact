//! Role: CpuRaster: rasterizes validated scenes with vello_cpu on the calling thread.
//!
//! Glyphs: every glyph is drawn from its outline, hinted or not. The GPU
//! draws hinted solid runs from glifo's glyph atlas, but the CPU does not:
//! vello_cpu 0.2 fixes each atlas page at 4096 × 4096 RGBA (64 MiB, up to
//! eight pages, not configurable), and every raster would hold that memory.
//! Measured at 1280 × 800 with 12,000 Inter glyphs over 100 frames, the
//! atlas drew a steady frame in 12–13 ms against 22–23 ms from outlines, but
//! its first frame took 34–42 ms against 24.5 ms, and most of this crate's
//! CPU rasters are short-lived. The atlas is revisited only once vello_cpu
//! exposes the atlas page size.

use crate::derived::{Derived, DerivedScope};
use crate::draw::{self, GlyphRun};
use crate::error::RenderError;
use crate::{images, paint, validate};
use arvane_protocol::{PixelRect, RenderScene};
use kivact_types::pixel::Pixels;
use kivact_types::{Point, Rect as LogicalRect};
use std::sync::Arc;
use vello_common::filter_effects::Filter;
use vello_cpu::kurbo::{Affine, BezPath, Rect, Stroke};
use vello_cpu::peniko::{
    self, BlendMode, Blob, Extend, ImageAlphaType, ImageFormat, ImageQuality, ImageSampler,
};
use vello_cpu::{
    Glyph, Image, ImageSource, PaintType, PixmapMut, RenderContext, RenderSettings, Resources,
};

/// CPU rasterizer, used serially on the thread which owns it.
///
/// The single-thread rasterizer and reusable resource cache are private.
/// It draws on the calling thread alone, so a scene's raster is the same
/// bit for bit on every run and machine. (vello_cpu's worker threads halve
/// the cost of simple frames but double that of text-heavy ones, so they
/// are not used.)
pub struct CpuRaster {
    context: RenderContext,
    resources: Resources,
    caches: draw::Caches,
    /// Pixels derived from the scenes' values, kept from frame to frame.
    derived: Derived,
    /// Premultiplied images of the last frame, by their pixels' identity.
    images: images::ImageCache<ImageSource>,
    /// The raster of a partial frame's region, copied out row by row.
    scratch: Vec<u8>,
}

impl Default for CpuRaster {
    fn default() -> Self {
        Self {
            context: RenderContext::new_with(
                1,
                1,
                RenderSettings {
                    level: vello_cpu::Level::new(),
                    num_threads: 0,
                },
            ),
            resources: Resources::new(),
            caches: draw::Caches::default(),
            derived: Derived::default(),
            images: images::ImageCache::new(images::MAX_BYTES),
            scratch: Vec::new(),
        }
    }
}

impl CpuRaster {
    /// Create an empty rasterizer and persistent raster resources.
    pub fn new() -> CpuRaster {
        Self::default()
    }

    /// Release rebuildable font validation metadata, retaining live drawing resources.
    pub fn release_caches(&mut self) {
        self.caches.validation.clear();
    }

    /// Release the partial-frame scratch; the next partial frame grows it again.
    pub fn forget_scratch(&mut self) {
        self.scratch = Vec::new();
    }

    /// Run `body` as one frame: every raster it makes shares the frame's
    /// image and derived-tile caches (AR-01).
    pub fn frame<R>(&mut self, body: impl FnOnce(&mut CpuFrameRaster<'_>) -> R) -> R {
        let CpuRaster {
            context,
            resources,
            caches,
            derived,
            images,
            scratch,
        } = self;
        caches.validation.prune();
        images.frame(|images| {
            derived.frame(|derived| {
                body(&mut CpuFrameRaster {
                    context,
                    resources,
                    caches,
                    images,
                    derived,
                    scratch,
                })
            })
        })
    }
}

/// One window frame of rasters. Images converted and tiles derived for
/// any raster of the frame stay cached for the next frame; what no raster
/// of the frame drew is dropped when the frame ends.
pub struct CpuFrameRaster<'a> {
    context: &'a mut RenderContext,
    resources: &'a mut Resources,
    caches: &'a mut draw::Caches,
    images: images::ImageScope<'a, ImageSource>,
    derived: DerivedScope<'a>,
    scratch: &'a mut Vec<u8>,
}

/// Where a scene is drawn: the logical point at the frame's top left, the
/// device pixels per logical pixel, and the frame's size in device pixels.
#[derive(Clone, Copy)]
struct Target {
    origin: Point,
    scale: f64,
    size: (u16, u16),
}

impl CpuFrameRaster<'_> {
    /// Draw `scene` (logical px, `origin` at the frame's top left, `scale`
    /// device px per logical px) into `out`, a premultiplied RGBA8 frame of
    /// `size` device px; only `clip` when given.
    pub fn raster(
        &mut self,
        scene: &RenderScene,
        origin: Point,
        scale: f64,
        size: (u16, u16),
        clip: Option<PixelRect>,
        out: &mut [u8],
    ) -> Result<(), RenderError> {
        let full = PixelRect {
            x: 0,
            y: 0,
            width: u32::from(size.0),
            height: u32::from(size.1),
        };
        let region = clip.unwrap_or(full);
        let target = Target {
            origin,
            scale,
            size,
        };
        self.render(scene, target, region, &[region], out)
    }

    /// Draw `scene` into the damaged `rects` of `out` with one validation
    /// and one encode (AR-04): the rects' bounding box is rendered once and
    /// only the rects are copied.
    pub fn raster_rects(
        &mut self,
        scene: &RenderScene,
        origin: Point,
        scale: f64,
        size: (u16, u16),
        rects: &[PixelRect],
        out: &mut [u8],
    ) -> Result<(), RenderError> {
        let Some(bounds) = bounding(rects) else {
            return Ok(());
        };
        let target = Target {
            origin,
            scale,
            size,
        };
        self.render(scene, target, bounds, rects, out)
    }

    /// Draw `scene` into `region` of the frame, then copy the `copy` rects
    /// of `region` into `out` (the whole frame, `target.size` device px).
    fn render(
        &mut self,
        scene: &RenderScene,
        target: Target,
        region: PixelRect,
        copy: &[PixelRect],
        out: &mut [u8],
    ) -> Result<(), RenderError> {
        let Target {
            origin,
            scale,
            size,
        } = target;
        if !scale.is_finite() || scale <= 0.0 {
            return Err(RenderError::InvalidSize);
        }
        validate::target(size)?;
        let (frame_width, frame_height) = (u32::from(size.0), u32::from(size.1));
        if out.len() != frame_width as usize * frame_height as usize * 4 {
            return Err(RenderError::InvalidSize);
        }
        if region.width == 0
            || region.height == 0
            || u64::from(region.x) + u64::from(region.width) > u64::from(frame_width)
            || u64::from(region.y) + u64::from(region.height) > u64::from(frame_height)
        {
            return Err(RenderError::InvalidSize);
        }
        // A scene drawn from the region's corner is the same as one drawn
        // from the frame's, moved by the region's origin.
        let at = Point {
            x: origin.x + f64::from(region.x) / scale,
            y: origin.y + f64::from(region.y) / scale,
        };
        let (width, height) = (region.width as u16, region.height as u16);
        let frame = draw::Frame {
            scale,
            origin: at,
            visible: LogicalRect {
                x: at.x,
                y: at.y,
                width: f64::from(width) / scale,
                height: f64::from(height) / scale,
            },
        };
        validate::scene(scene, &mut self.caches.validation)?;
        self.context.reset_and_resize(width, height);
        let mut canvas = CpuCanvas {
            context: &mut *self.context,
            resources: &mut *self.resources,
            images: &mut self.images,
        };
        draw::draw(
            &mut canvas,
            &mut *self.caches,
            &mut self.derived,
            scene,
            &frame,
        );
        self.context.flush();
        // A single rect that is the whole frame is drawn straight into `out`.
        // Its target is covered by the scene, so vello_cpu writes every pixel
        // without clearing first.
        if copy.len() == 1 && region.width == frame_width && region.height == frame_height {
            let target = PixmapMut::new(width, height, out).ok_or(RenderError::InvalidSize)?;
            self.context.render(target, &mut *self.resources);
            return Ok(());
        }
        let len = usize::from(width) * usize::from(height) * 4;
        self.scratch.resize(len, 0);
        let target = PixmapMut::new(width, height, &mut self.scratch[..len])
            .ok_or(RenderError::InvalidSize)?;
        self.context.render(target, &mut *self.resources);
        copy_rects(&self.scratch[..len], region, frame_width, copy, out);
        Ok(())
    }
}

/// Copy each of `copy` (rects inside `region`) from `scratch`, the region's
/// pixels, into `out`, the frame's pixels `frame_width` device px a row.
fn copy_rects(
    scratch: &[u8],
    region: PixelRect,
    frame_width: u32,
    copy: &[PixelRect],
    out: &mut [u8],
) {
    let region_stride = region.width as usize * 4;
    let frame_stride = frame_width as usize * 4;
    for rect in copy {
        let span = rect.width as usize * 4;
        let left = (rect.x - region.x) as usize * 4;
        let top = (rect.y - region.y) as usize;
        for row in 0..rect.height as usize {
            let from = (top + row) * region_stride + left;
            let to = (rect.y as usize + row) * frame_stride + rect.x as usize * 4;
            out[to..to + span].copy_from_slice(&scratch[from..from + span]);
        }
    }
}

/// The smallest rect holding every rect of `rects`; `None` when empty.
fn bounding(rects: &[PixelRect]) -> Option<PixelRect> {
    let mut iter = rects.iter();
    let first = *iter.next()?;
    let (mut x0, mut y0) = (first.x, first.y);
    let (mut x1, mut y1) = (first.x + first.width, first.y + first.height);
    for r in iter {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.width);
        y1 = y1.max(r.y + r.height);
    }
    Some(PixelRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

/// The CPU rasterizer's [`draw::Canvas`].
struct CpuCanvas<'a, 'b> {
    context: &'a mut RenderContext,
    resources: &'a mut Resources,
    images: &'a mut images::ImageScope<'b, ImageSource>,
}

impl CpuCanvas<'_, '_> {
    /// `image` premultiplied, kept while frames draw it.
    fn source(&mut self, image: &Pixels) -> ImageSource {
        // The pixels are premultiplied already, so vello_cpu takes them as
        // they are; its pixmap is still made once and the cache keeps it
        // while frames draw it.
        images::converted(self.images, image, || {
            ImageSource::from_peniko_image_data(&vello_cpu::peniko::ImageData {
                data: Blob::new(Arc::new(Arc::clone(image.rgba.as_shared()))),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::AlphaPremultiplied,
                width: image.width,
                height: image.height,
            })
        })
    }
}

impl draw::Canvas for CpuCanvas<'_, '_> {
    fn set_transform(&mut self, transform: Affine) {
        self.context.set_transform(transform);
    }

    fn set_paint(&mut self, paint: PaintType) {
        self.context.set_paint(paint);
    }

    fn set_paint_transform(&mut self, transform: Affine) {
        self.context.set_paint_transform(transform);
    }

    fn reset_paint_transform(&mut self) {
        self.context.reset_paint_transform();
    }

    fn set_fill_rule(&mut self, rule: peniko::Fill) {
        self.context.set_fill_rule(rule);
    }

    fn set_blend_mode(&mut self, mode: BlendMode) {
        self.context.set_blend_mode(mode);
    }

    fn set_stroke(&mut self, stroke: Stroke) {
        self.context.set_stroke(stroke);
    }

    fn fill_rect(&mut self, rect: &Rect) {
        self.context.fill_rect(rect);
    }

    fn fill_path(&mut self, path: &BezPath) {
        self.context.fill_path(path);
    }

    fn stroke_path(&mut self, path: &BezPath) {
        self.context.stroke_path(path);
    }

    fn fill_blurred_rounded_rect(&mut self, rect: &Rect, radius: f64, std_dev: f64) {
        self.context
            .fill_blurred_rounded_rect(rect, radius as f32, std_dev as f32, false);
    }

    fn push_clip_layer(&mut self, path: &BezPath) {
        self.context.push_clip_layer(path);
    }

    fn push_clip_path(&mut self, path: &BezPath) {
        self.context.push_clip_path(path);
    }

    fn pop_clip_path(&mut self) {
        self.context.pop_clip_path();
    }

    fn push_opacity_layer(&mut self, clip: &BezPath, opacity: f64) {
        self.context
            .push_layer(Some(clip), None, Some(opacity as f32), None, None);
    }

    fn push_layer(
        &mut self,
        clip: Option<&BezPath>,
        blend: Option<BlendMode>,
        opacity: Option<f64>,
        filter: Option<Filter>,
    ) {
        self.context
            .push_layer(clip, blend, opacity.map(|value| value as f32), None, filter);
    }

    fn pop_layer(&mut self) {
        self.context.pop_layer();
    }

    fn fill_glyphs(&mut self, run: GlyphRun<'_>, glyphs: impl Iterator<Item = Glyph> + Clone) {
        run.apply(self.context.glyph_run(self.resources, run.font))
            .fill_glyphs(glyphs);
    }

    fn stroke_glyphs(&mut self, run: GlyphRun<'_>, glyphs: impl Iterator<Item = Glyph> + Clone) {
        run.apply(self.context.glyph_run(self.resources, run.font))
            .stroke_glyphs(glyphs);
    }

    fn draw_image(&mut self, image: &Pixels, rect: Rect, pixelated: bool) {
        let source = self.source(image);
        self.context.set_paint(PaintType::Image(Image {
            image: source,
            sampler: ImageSampler {
                quality: if pixelated {
                    ImageQuality::Low
                } else {
                    ImageQuality::Medium
                },
                ..ImageSampler::default()
            },
        }));
        self.context.set_paint_transform(
            Affine::translate((rect.x0, rect.y0))
                * Affine::scale_non_uniform(
                    rect.width() / f64::from(image.width),
                    rect.height() / f64::from(image.height),
                ),
        );
        self.context.fill_rect(&rect);
        self.context.reset_paint_transform();
    }

    fn fill_pattern(&mut self, path: &BezPath, pattern: &paint::PatternFill, _visible: Rect) {
        let source = self.source(&pattern.image);
        let extend = |repeat: bool| if repeat { Extend::Repeat } else { Extend::Pad };
        self.context.set_paint(PaintType::Image(Image {
            image: source,
            sampler: ImageSampler {
                x_extend: extend(pattern.repeat_x),
                y_extend: extend(pattern.repeat_y),
                quality: ImageQuality::Medium,
                alpha: 1.0,
            },
        }));
        self.context.set_paint_transform(pattern.transform);
        self.context.fill_path(path);
        self.context.reset_paint_transform();
    }
}
