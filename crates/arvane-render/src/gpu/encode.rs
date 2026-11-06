//! Role: Encoding of Arvane scenes into vello_hybrid scenes for the GPU renderer.
//!
//! Arvane scenes as vello_hybrid scenes: the shared drawing (`draw.rs`)
//! into the GPU renderer's scene.
//!
//! Glyphs: drawn from vello_hybrid's glyph atlas (glifo's cache: each glyph
//! is rasterized once per size, hinting, emboldening and quarter-pixel
//! position into an atlas page and copied from there; pages of
//! [`GLYPH_PAGE`] pixels are added as needed and unused glyphs are evicted
//! after 64 frames), so a frame of text costs one textured rectangle per
//! glyph rather than its outline's strips. Glyphs under a rotation, skew
//! or non-uniform scale are drawn from their outlines.
//!
//! Images: each image gets its own texture (premultiplied RGBA8), made
//! once while consecutive frames draw it (the CPU renderer's image cache
//! rules, `images.rs`) and dropped after the first frame that does not
//! draw it. An image larger than the device's texture limit is shrunk to
//! fit first (by averaging blocks of pixels). A texture repeats its last
//! column and row once more (`upload`), so bilinear sampling, which
//! clamps before it moves half a texel back, samples the last texel as
//! the CPU does.
//!
//! Patterns: a pattern whose tile is at most [`PATTERN_ATLAS`] device
//! pixels on each side repeats from a slot of the image atlas (the glyph
//! atlas's pages; the shader repeats atlas images, not textures, which
//! have no extend), its image averaged down to that size first when
//! larger; the slot is kept while consecutive frames draw the pattern and
//! freed after the first that does not. A slot holds at most 256 KiB and
//! at most `MAX_ATLAS_PATTERNS` are in the atlas at once (16 MiB), since
//! vello_hybrid panics when its pages (64 at most) run out; a pattern
//! beyond that, and a larger tile, is drawn as one textured rectangle per
//! visible copy (few, as they are large; at most `MAX_TILES`, 4096, are
//! drawn). Atlas uploads and clears are
//! submitted apart, before the frame is drawn.
//!
//! Limits:
//! - The glyph atlas is a feature vello_hybrid calls experimental (its
//!   pages are allocated and evicted inside vello_hybrid and glifo). A
//!   glyph above 128 ppem does not use it and is drawn from its strips.
//! - In a frame that draws new glyphs into the atlas, vello_hybrid 0.2
//!   draws the atlas page (512 px) in a submission of its own and makes
//!   its depth texture again whenever the size it draws at changes: twice
//!   in a frame that goes between the atlas and the window. That is inside
//!   vello_hybrid and left as it is.
//! - An image beyond the device's texture limit (16,384 px on most) is
//!   shrunk, so it is coarser on the GPU than on the CPU.
//! - External frames (textures an application's render engine shares,
//!   `external.rs`) are sampled from the engine's own texture, which has
//!   no repeated last column and row: bilinear sampling of a scaled frame
//!   blends the last texel with the one before it within its last half
//!   texel (vello_hybrid clamps first), where an image samples the last
//!   texel alone. Drawn at its own size, a frame's pixels are exact.

use super::external::Frame;
use super::state::{AtlasImage, GpuContext, GpuImage, Images};
use crate::draw;
use crate::error::RenderError;
use crate::paint::PatternFill;
use crate::validate;
use crate::validate::SceneInfo;
use kivact_types::render::RenderScene;
use kivact_types::scene::Pixels;
use kivact_types::{ExternalFrame, Size};
use std::sync::Arc;
use vello_common::filter_effects::Filter;
use vello_common::geometry::RectU16;
use vello_common::paint::ImageId;
use vello_cpu::color::PremulRgba8;
use vello_cpu::color::palette::css::WHITE;
use vello_cpu::kurbo::{Affine, BezPath, Rect, Shape, Stroke};
use vello_cpu::peniko::{
    self, BlendMode, Blob, Extend, ImageAlphaType, ImageFormat, ImageQuality, ImageSampler,
};
use vello_cpu::{Glyph, Image, ImageSource, PaintType, Pixmap};
use vello_hybrid::{Renderer, Resources, SampleRect, TextureBindings, TextureId};

/// The side of a glyph atlas page, in pixels (1 MiB each): a page holds
/// about a thousand glyphs of body text.
pub(crate) const GLYPH_PAGE: u32 = 512;

/// The longest side, in device pixels, of a pattern tile repeated from the
/// image atlas (a larger one is drawn copy by copy).
const PATTERN_ATLAS: u16 = 256;

/// The most copies of a large pattern tile drawn as rectangles.
const MAX_TILES: usize = 4096;

/// The most pattern images in the atlas at once (16 MiB): the atlas is the
/// glyphs' too, and vello_hybrid panics when its pages run out.
const MAX_ATLAS_PATTERNS: usize = 64;

impl GpuContext {
    /// Record `source` into the renderer's scene for a `width` × `height`
    /// frame at `scale` device pixels per logical pixel, on white when
    /// `opaque` (as CPU frames are shown).
    pub(crate) fn encode(
        &mut self,
        source: &RenderScene,
        scale: f64,
        size: (u32, u32),
        info: &SceneInfo,
        opaque: bool,
        external: &Frame,
    ) {
        let Self {
            scene,
            resources,
            renderer,
            device,
            queue,
            images,
            caches,
            ..
        } = self;
        record(
            Recorder {
                scene,
                resources,
                renderer,
                device,
                queue,
                images,
                caches,
                external,
            },
            source,
            scale,
            size,
            info,
            opaque,
        );
    }

    /// [`GpuContext::encode`], while a scoped thread runs `alongside` with
    /// the device and the adapter (driver work that would otherwise wait
    /// for the recording, or the recording for it); returns what it
    /// returned.
    pub(crate) fn encode_alongside<R: Send>(
        &mut self,
        source: &RenderScene,
        scale: f64,
        size: (u32, u32),
        info: &SceneInfo,
        external: &Frame,
        alongside: impl FnOnce(&wgpu::Device, &wgpu::Adapter) -> R + Send,
    ) -> R {
        let Self {
            scene,
            resources,
            renderer,
            device,
            queue,
            images,
            caches,
            adapter,
            ..
        } = self;
        let (device, adapter) = (&*device, &*adapter);
        std::thread::scope(|scope| {
            let side = scope.spawn(move || alongside(device, adapter));
            record(
                Recorder {
                    scene,
                    resources,
                    renderer,
                    device,
                    queue,
                    images,
                    caches,
                    external,
                },
                source,
                scale,
                size,
                info,
                true,
            );
            side.join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
        })
    }
}

/// The context's parts a recording uses, borrowed apart.
struct Recorder<'a> {
    scene: &'a mut vello_hybrid::Scene,
    resources: &'a mut Resources,
    renderer: &'a mut Renderer,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    images: &'a mut Images,
    caches: &'a mut draw::Caches,
    external: &'a Frame,
}

/// Record `source` into the scene (see [`GpuContext::encode`]).
fn record(
    parts: Recorder<'_>,
    source: &RenderScene,
    scale: f64,
    (width, height): (u32, u32),
    info: &SceneInfo,
    opaque: bool,
) {
    let Recorder {
        scene,
        resources,
        renderer,
        device,
        queue,
        images,
        caches,
        external,
    } = parts;
    let (width, height) = (
        u16::try_from(width).expect("invariant: validated frame width"),
        u16::try_from(height).expect("invariant: validated frame height"),
    );
    if (scene.width(), scene.height()) == (width, height) {
        scene.reset();
    } else {
        scene.reset_and_resize(width, height);
    }
    if opaque {
        scene.set_paint(PaintType::Solid(WHITE));
        scene.fill_rect(&Rect::new(0.0, 0.0, f64::from(width), f64::from(height)));
    }
    images.cache.begin_frame();
    images.bindings = TextureBindings::new();
    let mut canvas = HybridCanvas {
        scene,
        resources,
        renderer,
        device,
        queue,
        images,
        external,
        uploads: None,
    };
    let frame = draw::Frame {
        scale,
        size: Size {
            width: f64::from(width) / scale,
            height: f64::from(height) / scale,
        },
        backdrops: info.backdrops,
    };
    draw::draw(&mut canvas, caches, source, &frame);
    // Dropping an image's view frees its texture once the GPU is done.
    canvas.images.cache.end_frame();
    canvas.free_stale_patterns();
    if let Some(uploads) = canvas.uploads.take() {
        queue.submit([uploads.finish()]);
    }
}

/// The GPU renderer's [`draw::Canvas`].
struct HybridCanvas<'a> {
    scene: &'a mut vello_hybrid::Scene,
    resources: &'a mut Resources,
    renderer: &'a mut Renderer,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    images: &'a mut Images,
    /// The external frames ready for this frame.
    external: &'a Frame,
    /// Atlas uploads and clears of this recording, submitted before the
    /// frame is drawn.
    uploads: Option<wgpu::CommandEncoder>,
}

impl HybridCanvas<'_> {
    fn uploads(&mut self) -> &mut wgpu::CommandEncoder {
        self.uploads.get_or_insert_with(|| {
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Arvane atlas uploads"),
                })
        })
    }

    /// `image`'s texture, bound for this frame.
    fn texture(&mut self, image: &Pixels) -> GpuImage {
        let (device, queue, next) = (self.device, self.queue, &mut self.images.next);
        let texture = self.images.cache.get(image, || {
            *next += 1;
            upload(device, queue, TextureId(*next), image)
        });
        self.images
            .bindings
            .insert(texture.id, texture.view.clone());
        texture
    }

    /// `image`'s atlas slot (averaged down to [`PATTERN_ATLAS`] pixels a
    /// side when larger): id and pixel size.
    fn atlas(&mut self, image: &Pixels) -> (ImageId, u16, u16) {
        let key = (
            Arc::as_ptr(image.rgba.as_shared()).cast::<u8>() as usize,
            image.width,
            image.height,
        );
        let frame = self.images.cache.frame();
        if let Some(slot) = self.images.atlas.get_mut(&key) {
            slot.frame = frame;
            return (slot.id, slot.width, slot.height);
        }
        let pixmap = shrink(premultiplied(image), PATTERN_ATLAS);
        let (width, height) = (pixmap.width(), pixmap.height());
        let (device, queue) = (self.device, self.queue);
        let encoder = self.uploads.get_or_insert_with(|| {
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane atlas uploads"),
            })
        });
        let id = self
            .renderer
            .upload_image(self.resources, device, queue, encoder, &pixmap);
        self.images.atlas.insert(
            key,
            AtlasImage {
                _pixels: Arc::clone(image.rgba.as_shared()),
                id,
                width,
                height,
                frame,
            },
        );
        (id, width, height)
    }

    /// Free the atlas slots of patterns this frame did not draw.
    fn free_stale_patterns(&mut self) {
        let frame = self.images.cache.frame();
        let stale: Vec<ImageId> = self
            .images
            .atlas
            .iter()
            .filter(|(_, slot)| slot.frame != frame)
            .map(|(_, slot)| slot.id)
            .collect();
        if stale.is_empty() {
            return;
        }
        self.images.atlas.retain(|_, slot| slot.frame == frame);
        self.uploads();
        let encoder = self.uploads.as_mut().expect("invariant: just made");
        for id in stale {
            self.renderer.destroy_image(self.resources, encoder, id);
        }
    }
}

impl draw::Canvas for HybridCanvas<'_> {
    fn set_transform(&mut self, transform: Affine) {
        self.scene.set_transform(transform);
    }

    fn set_paint(&mut self, paint: PaintType) {
        self.scene.set_paint(paint);
    }

    fn set_paint_transform(&mut self, transform: Affine) {
        self.scene.set_paint_transform(transform);
    }

    fn reset_paint_transform(&mut self) {
        self.scene.reset_paint_transform();
    }

    fn set_stroke(&mut self, stroke: Stroke) {
        self.scene.set_stroke(stroke);
    }

    fn set_fill_rule(&mut self, rule: peniko::Fill) {
        self.scene.set_fill_rule(rule);
    }

    fn set_blend_mode(&mut self, mode: BlendMode) {
        self.scene.set_blend_mode(mode);
    }

    fn fill_rect(&mut self, rect: &Rect) {
        self.scene.fill_rect(rect);
    }

    fn fill_path(&mut self, path: &BezPath) {
        self.scene.fill_path(path);
    }

    fn stroke_path(&mut self, path: &BezPath) {
        self.scene.stroke_path(path);
    }

    fn fill_blurred_rounded_rect(&mut self, rect: &Rect, radius: f64, std_dev: f64) {
        self.scene
            .fill_blurred_rounded_rect(rect, radius as f32, std_dev as f32, false);
    }

    fn push_clip_layer(&mut self, path: &BezPath) {
        self.scene.push_clip_layer(path);
    }

    fn push_clip_path(&mut self, path: &BezPath) {
        self.scene.push_clip_path(path);
    }

    fn pop_clip_path(&mut self) {
        self.scene.pop_clip_path();
    }

    fn push_opacity_layer(&mut self, clip: &BezPath, opacity: f64) {
        self.scene
            .push_layer(Some(clip), None, Some(opacity as f32), None, None);
    }

    fn push_layer(
        &mut self,
        clip: Option<&BezPath>,
        blend: Option<BlendMode>,
        opacity: Option<f64>,
        filter: Option<Filter>,
    ) {
        self.scene
            .push_layer(clip, blend, opacity.map(|value| value as f32), None, filter);
    }

    fn pop_layer(&mut self) {
        self.scene.pop_layer();
    }

    fn fill_glyphs(
        &mut self,
        run: draw::GlyphRun<'_>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        run.apply(self.scene.glyph_run(self.resources, run.font))
            .atlas_cache(run.cached)
            .fill_glyphs(glyphs);
    }

    fn draw_image(&mut self, image: &Pixels, rect: Rect, pixelated: bool) {
        let texture = self.texture(image);
        // The texture's extra column and row (see `upload`) are sampled
        // but not drawn.
        self.scene.push_clip_path(&rect.to_path(0.1));
        self.scene.draw_texture_rects(
            texture.id,
            if pixelated {
                ImageQuality::Low
            } else {
                ImageQuality::Medium
            },
            [SampleRect {
                source_region: RectU16::new(0, 0, texture.width + 1, texture.height + 1),
                transform: Affine::translate((rect.x0, rect.y0))
                    * Affine::scale_non_uniform(
                        rect.width() / f64::from(texture.width),
                        rect.height() / f64::from(texture.height),
                    ),
            }],
        );
        self.scene.pop_clip_path();
    }

    fn external_size(&self, frame: &ExternalFrame) -> Option<(u32, u32)> {
        self.external
            .bound(frame)
            .map(|bound| (u32::from(bound.width), u32::from(bound.height)))
    }

    fn draw_external(&mut self, frame: &ExternalFrame, rect: Rect, pixelated: bool) {
        let Some(bound) = self.external.bound(frame) else {
            return;
        };
        self.images.bindings.insert(bound.id, bound.view.clone());
        self.scene.push_clip_path(&rect.to_path(0.1));
        // The whole texture: it has no repeated last column and row (see
        // the module's limits).
        self.scene.draw_texture_rects(
            bound.id,
            if pixelated {
                ImageQuality::Low
            } else {
                ImageQuality::Medium
            },
            [SampleRect {
                source_region: RectU16::new(0, 0, bound.width, bound.height),
                transform: Affine::translate((rect.x0, rect.y0))
                    * Affine::scale_non_uniform(
                        rect.width() / f64::from(bound.width),
                        rect.height() / f64::from(bound.height),
                    ),
            }],
        );
        self.scene.pop_clip_path();
    }

    fn fill_pattern(&mut self, path: &BezPath, pattern: &PatternFill, visible: Rect) {
        let small = f64::from(PATTERN_ATLAS);
        let key = (
            Arc::as_ptr(pattern.image.rgba.as_shared()).cast::<u8>() as usize,
            pattern.image.width,
            pattern.image.height,
        );
        let room =
            self.images.atlas.contains_key(&key) || self.images.atlas.len() < MAX_ATLAS_PATTERNS;
        if pattern.device_tile.0 <= small && pattern.device_tile.1 <= small && room {
            let (id, width, height) = self.atlas(&pattern.image);
            let extend = |repeat: bool| if repeat { Extend::Repeat } else { Extend::Pad };
            self.scene.set_paint(PaintType::Image(Image {
                image: ImageSource::opaque_id(id),
                sampler: ImageSampler {
                    x_extend: extend(pattern.repeat_x),
                    y_extend: extend(pattern.repeat_y),
                    quality: ImageQuality::Medium,
                    alpha: 1.0,
                },
            }));
            // The slot may be smaller than the image.
            self.scene.set_paint_transform(
                pattern.transform
                    * Affine::scale_non_uniform(
                        f64::from(pattern.image.width) / f64::from(width),
                        f64::from(pattern.image.height) / f64::from(height),
                    ),
            );
            self.scene.fill_path(path);
            self.scene.reset_paint_transform();
            return;
        }
        // Large tiles: one rectangle per visible copy.
        let texture = self.texture(&pattern.image);
        let area = path.bounding_box().intersect(visible);
        if area.is_zero_area() {
            return;
        }
        let tile = pattern.tile;
        let (tx, ty, tw, th) = (tile.x, tile.y, tile.width, tile.height);
        let span = |repeat: bool, from: f64, to: f64, origin: f64, size: f64| {
            if repeat {
                (
                    ((from - origin) / size).floor() as i64,
                    ((to - origin) / size).ceil() as i64,
                )
            } else {
                (0, 1)
            }
        };
        let (i0, i1) = span(pattern.repeat_x, area.x0, area.x1, tx, tw);
        let (j0, j1) = span(pattern.repeat_y, area.y0, area.y1, ty, th);
        let rects: Vec<SampleRect> = (j0..j1)
            .flat_map(|j| (i0..i1).map(move |i| (i, j)))
            .take(MAX_TILES)
            .map(|(i, j)| SampleRect {
                source_region: RectU16::new(0, 0, texture.width, texture.height),
                transform: Affine::translate((tx + i as f64 * tw, ty + j as f64 * th))
                    * Affine::scale_non_uniform(
                        tw / f64::from(texture.width),
                        th / f64::from(texture.height),
                    ),
            })
            .collect();
        self.scene.push_clip_path(path);
        self.scene
            .draw_texture_rects(texture.id, ImageQuality::Medium, rects);
        self.scene.pop_clip_path();
    }
}

/// A texture of `image`'s premultiplied pixels, with its last column and
/// row repeated once more.
///
/// vello_hybrid's bilinear sampling clamps the position to the last texel
/// before moving it half a texel back to the texel grid, so the last half
/// texel of an image would blend the last column (row) with the one
/// before it. With the repeated column (row) inside the sampled region,
/// the clamp no longer applies there and the last texel is sampled as the
/// CPU samples it.
fn upload(device: &wgpu::Device, queue: &wgpu::Queue, id: TextureId, image: &Pixels) -> GpuImage {
    let limit = device
        .limits()
        .max_texture_dimension_2d
        .min(u32::from(u16::MAX)) as u16
        - 1;
    let pixmap = shrink(premultiplied(image), limit);
    let (width, height) = (usize::from(pixmap.width()), usize::from(pixmap.height()));
    let mut padded = Vec::with_capacity((width + 1) * (height + 1) * 4);
    let rows = pixmap.data_as_u8_slice().chunks_exact(width * 4);
    for row in rows.clone().chain(rows.last()) {
        padded.extend_from_slice(row);
        padded.extend_from_slice(&row[(width - 1) * 4..]);
    }
    let size = wgpu::Extent3d {
        width: width as u32 + 1,
        height: height as u32 + 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Arvane image"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &padded,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.width),
            rows_per_image: Some(size.height),
        },
        size,
    );
    GpuImage {
        id,
        view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
        width: pixmap.width(),
        height: pixmap.height(),
    }
}

/// `image`'s pixels, premultiplied.
fn premultiplied(image: &Pixels) -> Arc<Pixmap> {
    let source = ImageSource::from_peniko_image_data(&vello_cpu::peniko::ImageData {
        data: Blob::new(Arc::new(Arc::clone(image.rgba.as_shared()))),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width: image.width,
        height: image.height,
    });
    match source {
        ImageSource::Pixmap(pixmap) => pixmap,
        ImageSource::OpaqueId { .. } => unreachable!("pixels convert to a pixmap"),
    }
}

/// `pixmap`, or when a side is longer than `limit`, the average of each
/// block of n × n pixels (the smallest n that fits).
fn shrink(pixmap: Arc<Pixmap>, limit: u16) -> Arc<Pixmap> {
    let (width, height) = (pixmap.width(), pixmap.height());
    let n = width.max(height).div_ceil(limit);
    if n <= 1 {
        return pixmap;
    }
    let (to_width, to_height) = (width.div_ceil(n), height.div_ceil(n));
    let data = pixmap.data();
    let mut out = Vec::with_capacity(usize::from(to_width) * usize::from(to_height));
    for y in 0..to_height {
        for x in 0..to_width {
            let mut sum = [0u32; 4];
            let mut count = 0;
            for sy in y * n..((y + 1) * n).min(height) {
                for sx in x * n..((x + 1) * n).min(width) {
                    let p = data[usize::from(sy) * usize::from(width) + usize::from(sx)];
                    for (s, c) in sum.iter_mut().zip([p.r, p.g, p.b, p.a]) {
                        *s += u32::from(c);
                    }
                    count += 1;
                }
            }
            let [r, g, b, a] = sum.map(|s| ((s + count / 2) / count) as u8);
            out.push(PremulRgba8 { r, g, b, a });
        }
    }
    Arc::new(Pixmap::from_parts(out, to_width, to_height))
}

impl GpuContext {
    /// Validate `scene` and the frame size against the renderer's budget
    /// and the device's texture limit, returning the pixel dimensions and
    /// what drawing the scene needs to know.
    pub(crate) fn dimensions(
        &mut self,
        scene: &RenderScene,
        size: Size,
        scale: f64,
    ) -> Result<((u32, u32), validate::SceneInfo), RenderError> {
        let (width, height) = validate::dimensions(size, scale)?;
        let info = validate::scene(scene, &mut self.caches.validation)?;
        let (width, height) = (u32::from(width), u32::from(height));
        let max = self.device.limits().max_texture_dimension_2d;
        if width > max || height > max {
            return Err(RenderError::InvalidSize);
        }
        self.check()?;
        Ok(((width, height), info))
    }

    /// Draw the recorded scene into `view` (a `width` × `height` texture
    /// in [`FORMAT`]) with `encoder`.
    pub(crate) fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        (width, height): (u32, u32),
        view: &wgpu::TextureView,
    ) -> Result<(), RenderError> {
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                &self.device,
                &self.queue,
                encoder,
                &vello_hybrid::RenderSize { width, height },
                view,
                &self.images.bindings,
            )
            .map_err(|e| match e {
                // What the scene asks of the renderer's atlases and
                // textures, not a device failure.
                vello_hybrid::RenderError::AtlasError(_)
                | vello_hybrid::RenderError::MissingTextureBinding(_)
                | vello_hybrid::RenderError::IntermediateTexture(_) => {
                    RenderError::GpuLimit(e.to_string())
                }
            })
    }
}
