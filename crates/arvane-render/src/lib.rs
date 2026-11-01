//! Role: Arvane render: rasterises `kivact-types` draw commands.
//!
//! Responsibility: scene rasterization with private vello_cpu and
//! vello_hybrid backends, which share every stage but the last (`draw.rs`).
//! The CPU path is deterministic and backs the tests' frames; the GPU path
//! (feature `gpu`) presents to window surfaces.
//! Images arrive decoded, as straight-alpha RGBA8
//! `ImageData`; this crate reads no encoded image. The
//! CPU renderer keeps an image's premultiplied copy while consecutive
//! frames draw it (`images`), so a photo left on screen is not converted
//! again every frame; pixels made from a scene's values (images through a
//! color matrix, pattern tiles at their device size) are kept likewise
//! (`derived`). What vello_hybrid lacks (mask layers, destructive
//! blending, color matrix filters, reading pixels back) is drawn in forms
//! both rasterizers have (`draw/effects.rs`), and a scene with backdrops
//! is indexed by where its commands draw (`backdrop`). With the feature
//! `gpu-interop`, the GPU path also draws the textures an application's
//! render engine shares (`DrawCommand::External`, `gpu/external.rs`),
//! imported through `kivact-gpu-share`, which holds the `unsafe` below
//! wgpu that needs; the CPU path never draws them.
//!
//! Dependencies: vello_hybrid 0.2 uses the same vello_common 0.2 and
//! glifo 0.3 as vello_cpu 0.2 (no vello_hybrid matches vello_cpu 0.3 yet).
//! vello_common supplies the types vello_cpu and vello_hybrid do not
//! re-export: the filters (blur, drop shadow), texture rectangles and image
//! atlas ids; glifo supplies the glyph settings both take but do not
//! re-export. wgpu is the version vello_hybrid uses, built with Vulkan,
//! DirectX 12 and Metal only; raw-window-handle is how a GPU surface takes
//! its window; bytemuck views softbuffer's buffer as vello_cpu's bytes.
//!
//! Invariant: renderer types never appear in the public API.

//! Successful font-face metadata is retained per renderer with weak byte
//! identity, face index and a 256-entry LRU bound. Every scene still checks
//! glyph ids, variation values, colors and composed placement coordinates.
//! `release_caches` drops only rebuildable validation metadata, preserving
//! imports, leases and drawing resources already in use.
#![forbid(unsafe_code)]

mod backdrop;
mod error;
mod gradient;
pub use error::RenderError;
mod convert;
mod derived;
mod draw;
#[cfg(feature = "gpu")]
pub(crate) mod gpu;
mod images;
mod ink;
mod limits;
mod paint;
mod text;
mod validate;

#[cfg(feature = "gpu")]
pub use gpu::state::{GpuContext, GpuEnv, GpuSurface, Presentation};
pub use ink::glyph_ink_bounds;
pub use validate::{dimensions, raster_plan};

pub use limits::MAX_SCENE_DEPTH;

use kivact_types::{CpuFrame, ImageData, Scene, Size};
use std::sync::Arc;
use vello_common::filter_effects::Filter;
use vello_cpu::kurbo::{Affine, BezPath, Rect, Stroke};
use vello_cpu::peniko::{
    self, BlendMode, Blob, Extend, ImageAlphaType, ImageFormat, ImageQuality, ImageSampler,
};
use vello_cpu::{
    Glyph, Image, ImageSource, PaintType, Pixmap, RenderContext, RenderSettings, Resources,
};

/// CPU renderer, used serially on the thread which owns it.
///
/// The single-thread rasterizer and reusable resource cache are private.
/// The same renderer serves native windows and deterministic headless
/// image tests. (vello_cpu's worker threads halve the cost of simple
/// frames but double that of text-heavy ones, so they are not used.) It
/// draws on the calling thread alone, so a scene's frame is the same bit
/// for bit on every run and machine.
pub struct CpuRenderer {
    context: RenderContext,
    resources: Resources,
    caches: draw::Caches,
    /// Premultiplied images of the last frame, by their pixels' identity.
    images: images::ImageCache<ImageSource>,
    /// The raster of a frame that is not drawn straight into its target,
    /// kept from frame to frame.
    pixmap: Pixmap,
}

impl Default for CpuRenderer {
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
            pixmap: Pixmap::new(1, 1),
        }
    }
}

impl CpuRenderer {
    /// Release rebuildable font validation metadata, retaining live drawing resources.
    pub fn release_caches(&mut self) {
        self.caches.validation.clear();
    }

    /// Create an empty renderer and persistent raster resources.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders a scene into `out`, a `width` × `height` buffer of
    /// 0x00RRGGBB pixels (premultiplied color composited over white, as
    /// [`CpuFrame::to_softbuffer`]), without an intermediate frame. A
    /// scene rasterized smaller than `out` (a scale reduced to the frame
    /// budget) is enlarged by nearest neighbor.
    pub fn render_0rgb(
        &mut self,
        scene: &Scene,
        size: Size,
        scale: f32,
        (width, height): (u32, u32),
        out: &mut [u32],
    ) -> Result<(), RenderError> {
        if out.len() != width as usize * height as usize {
            return Err(RenderError::InvalidSize);
        }
        let (w, h) = self.draw(scene, size, scale)?;
        if (u32::from(w), u32::from(h)) == (width, height) {
            let target = vello_cpu::PixmapMut::new(w, h, bytemuck::cast_slice_mut(out))
                .expect("invariant: the buffer holds width × height pixels");
            self.context.render(target, &mut self.resources);
            for pixel in out.iter_mut() {
                *pixel = over_white(pixel.to_le_bytes());
            }
            return Ok(());
        }
        self.pixmap.resize(w, h);
        self.context.render(&mut self.pixmap, &mut self.resources);
        let source = self.pixmap.data_as_u8_slice().as_chunks::<4>().0;
        let index = |target: u32, extent: u32, source: u16| {
            ((u64::from(target) * u64::from(source) / u64::from(extent)) as usize)
                .min(usize::from(source) - 1)
        };
        for (y, row) in out.chunks_exact_mut(width as usize).enumerate() {
            let line = index(y as u32, height, h) * usize::from(w);
            for (x, pixel) in row.iter_mut().enumerate() {
                *pixel = over_white(source[line + index(x as u32, width, w)]);
            }
        }
        Ok(())
    }

    /// Renders a scene using logical dimensions and a physical display scale.
    ///
    /// Coordinates and paint are validated before touching the render context,
    /// so an error leaves the renderer usable for the next frame.
    pub fn render(
        &mut self,
        scene: &Scene,
        size: Size,
        scale: f32,
    ) -> Result<CpuFrame, RenderError> {
        let (width, height) = self.draw(scene, size, scale)?;
        self.pixmap.resize(width, height);
        self.context.render(&mut self.pixmap, &mut self.resources);
        Ok(CpuFrame {
            width: u32::from(width),
            height: u32::from(height),
            rgba: self.pixmap.data_as_u8_slice().to_vec(),
        })
    }

    /// Validate `scene` and record it in the render context for a frame of
    /// `size` × `scale`, returning the frame's pixel dimensions.
    fn draw(&mut self, scene: &Scene, size: Size, scale: f32) -> Result<(u16, u16), RenderError> {
        let (width, height) = validate::dimensions(size, scale)?;
        let info = validate::scene(scene, &mut self.caches.validation)?;
        self.context.reset_and_resize(width, height);
        self.images.begin_frame();
        let mut canvas = CpuCanvas {
            context: &mut self.context,
            resources: &mut self.resources,
            images: &mut self.images,
        };
        let frame = draw::Frame {
            scale,
            size,
            backdrops: info.backdrops,
        };
        draw::draw(&mut canvas, &mut self.caches, scene, &frame);
        self.images.end_frame();
        self.context.flush();
        Ok((width, height))
    }
}

/// The CPU renderer's [`draw::Canvas`].
struct CpuCanvas<'a> {
    context: &'a mut RenderContext,
    resources: &'a mut Resources,
    images: &'a mut images::ImageCache<ImageSource>,
}

impl CpuCanvas<'_> {
    /// `image` premultiplied, kept while frames draw it.
    fn source(&mut self, image: &ImageData) -> ImageSource {
        // vello_cpu premultiplies into a new pixmap for each paint; the
        // cache keeps it while frames draw it.
        self.images.get(image, || {
            ImageSource::from_peniko_image_data(&vello_cpu::peniko::ImageData {
                data: Blob::new(Arc::new(Arc::clone(&image.rgba))),
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

    fn fill_blurred_rounded_rect(&mut self, rect: &Rect, radius: f32, std_dev: f32) {
        self.context
            .fill_blurred_rounded_rect(rect, radius, std_dev, false);
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

    fn push_opacity_layer(&mut self, clip: &BezPath, opacity: f32) {
        self.context
            .push_layer(Some(clip), None, Some(opacity), None, None);
    }

    fn push_layer(
        &mut self,
        clip: Option<&BezPath>,
        blend: Option<BlendMode>,
        opacity: Option<f32>,
        filter: Option<Filter>,
    ) {
        self.context.push_layer(clip, blend, opacity, None, filter);
    }

    fn pop_layer(&mut self) {
        self.context.pop_layer();
    }

    fn fill_glyphs(
        &mut self,
        run: draw::GlyphRun<'_>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        run.apply(self.context.glyph_run(self.resources, run.font))
            .fill_glyphs(glyphs);
    }

    fn draw_image(&mut self, image: &ImageData, rect: Rect, pixelated: bool) {
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

    /// The CPU renderer has no external frames: nothing is drawn.
    fn external_size(&self, _: &kivact_types::ExternalFrame) -> Option<(u32, u32)> {
        None
    }

    fn draw_external(&mut self, _: &kivact_types::ExternalFrame, _: Rect, _: bool) {}

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

/// A premultiplied RGBA pixel composited over white, as 0x00RRGGBB.
fn over_white([r, g, b, a]: [u8; 4]) -> u32 {
    let white = 255 - u32::from(a);
    let channel = |c: u8| (u32::from(c) + white).min(255);
    (channel(r) << 16) | (channel(g) << 8) | channel(b)
}
