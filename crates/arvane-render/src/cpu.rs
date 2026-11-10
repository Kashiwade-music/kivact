//! Role: CpuRaster: rasterizes validated scenes with vello_cpu on the calling thread.

use crate::draw::{self, GlyphRun};
use crate::error::RenderError;
use crate::{images, paint, validate};
use kivact_types::pixel::Pixels;
use kivact_types::render::{PixelRect, RenderScene};
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
            images: images::ImageCache::new(),
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

    /// Rasterize `scene` mapped by scale(scale) · translate(−origin) into
    /// `out` (size.0 × size.1 premultiplied RGBA8, row-major, no padding).
    /// With `clip`, only that rect of `out` is drawn and the rest is left as
    /// it is; without it `out` is cleared to transparent first.
    pub fn raster(
        &mut self,
        scene: &RenderScene,
        origin: Point,
        scale: f64,
        size: (u16, u16),
        clip: Option<PixelRect>,
        out: &mut [u8],
    ) -> Result<(), RenderError> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(RenderError::InvalidSize);
        }
        validate::target(size)?;
        let (frame_width, frame_height) = (u32::from(size.0), u32::from(size.1));
        if out.len() != frame_width as usize * frame_height as usize * 4 {
            return Err(RenderError::InvalidSize);
        }
        // The rectangle drawn: the whole frame, or `clip` within it.
        let region = clip.unwrap_or(PixelRect {
            x: 0,
            y: 0,
            width: frame_width,
            height: frame_height,
        });
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
        self.images.begin_frame();
        let mut canvas = CpuCanvas {
            context: &mut self.context,
            resources: &mut self.resources,
            images: &mut self.images,
        };
        draw::draw(&mut canvas, &mut self.caches, scene, &frame);
        self.images.end_frame();
        self.context.flush();
        // A whole frame is drawn straight into `out`. Its target is covered by
        // the scene, so vello_cpu writes every pixel without clearing first.
        if region.width == frame_width && region.height == frame_height {
            let target = PixmapMut::new(width, height, out).ok_or(RenderError::InvalidSize)?;
            self.context.render(target, &mut self.resources);
            return Ok(());
        }
        let len = usize::from(width) * usize::from(height) * 4;
        self.scratch.resize(len, 0);
        let target = PixmapMut::new(width, height, &mut self.scratch[..len])
            .ok_or(RenderError::InvalidSize)?;
        self.context.render(target, &mut self.resources);
        let row_bytes = usize::from(width) * 4;
        for (y, row) in self.scratch[..len].chunks_exact(row_bytes).enumerate() {
            let start = ((region.y as usize + y) * frame_width as usize + region.x as usize) * 4;
            out[start..start + row_bytes].copy_from_slice(row);
        }
        Ok(())
    }
}

/// The CPU rasterizer's [`draw::Canvas`].
struct CpuCanvas<'a> {
    context: &'a mut RenderContext,
    resources: &'a mut Resources,
    images: &'a mut images::ImageCache<ImageSource>,
}

impl CpuCanvas<'_> {
    /// `image` premultiplied, kept while frames draw it.
    fn source(&mut self, image: &Pixels) -> ImageSource {
        // vello_cpu premultiplies into a new pixmap for each paint; the
        // cache keeps it while frames draw it.
        self.images.get(image, || {
            ImageSource::from_peniko_image_data(&vello_cpu::peniko::ImageData {
                data: Blob::new(Arc::new(Arc::clone(image.rgba.as_shared()))),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width: image.width,
                height: image.height,
            })
        })
    }
}

impl draw::Canvas for CpuCanvas<'_> {
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
