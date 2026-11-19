//! Role: GPU rasterizer state: the renderer, its resources and the frame drawn.
//!
//! [`GpuRaster`] is the aggregate and [`GpuFrameRaster`] is one window frame
//! of it. Recording a scene into the renderer's scene is a method of the
//! frame type in `encode.rs`.

use super::images::{AtlasImage, GpuImage, Images, MAX_ATLAS_PATTERNS};
use crate::derived::DerivedScope;
use crate::draw;
use crate::error::RenderError;
use crate::images::{ImageCache, ImageScope, MAX_BYTES};
use crate::validate;
use arvane_protocol::RenderScene;
use kivact_types::{Point, Rect as LogicalRect};
use vello_common::paint::ImageId;
use vello_hybrid::{
    AtlasConfig, MemorySettings, RenderSettings, RenderTargetConfig, Renderer, Resources,
};

/// The format a GPU raster is drawn in (premultiplied RGBA8, unorm).
pub const RASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Glyph atlas pages at most (added as needed, `encode.rs`).
const GLYPH_PAGES: usize = 64;

/// The side of a glyph atlas page, in pixels (1 MiB each): a page holds
/// about a thousand glyphs of body text.
const GLYPH_PAGE: u32 = 512;

/// Where a scene is drawn: a view of `size` px whose top left corner is the
/// logical point `origin`, at `scale` device px per logical px.
#[derive(Clone, Copy, Debug)]
pub struct Canvas {
    /// The logical point at the view's top left corner.
    pub origin: Point,
    /// Device px per logical px.
    pub scale: f64,
    /// The view's size in px.
    pub size: (u16, u16),
}

/// The GPU rasterizer: a vello_hybrid renderer and its resources, drawing
/// on the device a caller gives each call.
pub struct GpuRaster {
    pub(super) renderer: Renderer,
    pub(super) resources: Resources,
    /// The scene the frames are recorded into, kept for its allocations.
    pub(super) scene: vello_hybrid::Scene,
    pub(super) caches: draw::Caches,
    /// Pixels derived from the scene's values, kept from frame to frame.
    pub(super) derived: crate::derived::Derived,
    /// The bindings of the last frame's images.
    pub(super) images: Images,
    /// The textures of the last frame's images, kept while frames draw them.
    pub(super) textures: ImageCache<GpuImage>,
    /// The pattern atlas slots of the last frame's patterns, at most
    /// `MAX_ATLAS_PATTERNS`.
    pub(super) atlas: ImageCache<AtlasImage>,
}

/// One window frame of GPU rasters: the image textures, atlas slots and
/// derived tiles that any raster of the frame draws stay for the next
/// frame (AR-01). Made by [`GpuRaster::frame`].
pub struct GpuFrameRaster<'a> {
    pub(super) device: &'a wgpu::Device,
    pub(super) queue: &'a wgpu::Queue,
    pub(super) renderer: &'a mut Renderer,
    pub(super) resources: &'a mut Resources,
    pub(super) scene: &'a mut vello_hybrid::Scene,
    pub(super) caches: &'a mut draw::Caches,
    pub(super) images: &'a mut Images,
    pub(super) textures: ImageScope<'a, GpuImage>,
    pub(super) atlas: ImageScope<'a, AtlasImage>,
    pub(super) derived: DerivedScope<'a>,
}

impl GpuRaster {
    /// Creates the renderer's pipelines on `device`, and draws one small
    /// blank frame so the renderer's first buffers are made here, not in
    /// the first frame of a scene.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<GpuRaster, RenderError> {
        let (renderer, resources) = Renderer::new_with(
            device,
            &RenderTargetConfig {
                format: RASTER_FORMAT,
                width: 1,
                height: 1,
            },
            render_settings(),
        );
        let mut raster = Self {
            renderer,
            resources,
            scene: vello_hybrid::Scene::new(1, 1),
            caches: draw::Caches::default(),
            derived: crate::derived::Derived::default(),
            images: Images::new(),
            textures: ImageCache::new(MAX_BYTES),
            atlas: ImageCache::new(MAX_ATLAS_PATTERNS),
        };
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Arvane raster warm-up"),
            size: wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: RASTER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Arvane raster warm-up"),
        });
        let canvas = Canvas {
            origin: Point { x: 0.0, y: 0.0 },
            scale: 1.0,
            size: (16, 16),
        };
        raster.frame(device, queue, |frame| {
            frame.raster(&mut encoder, &RenderScene::default(), canvas, &view)
        })?;
        queue.submit([encoder.finish()]);
        Ok(raster)
    }

    /// Release rebuildable validation metadata without releasing imports or leases.
    pub fn release_caches(&mut self) {
        self.caches.validation.clear();
    }

    /// Run `body` as one frame (AR-01). Atlas slots no raster of the
    /// frame drew are destroyed after it, in one submission.
    pub fn frame<R>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        body: impl FnOnce(&mut GpuFrameRaster<'_>) -> R,
    ) -> R {
        let GpuRaster {
            renderer,
            resources,
            scene,
            caches,
            derived,
            images,
            textures,
            atlas,
        } = self;
        caches.validation.prune();
        let mut evicted = Vec::new();
        let result = atlas.frame_evicting(
            |atlas| {
                textures.frame(|textures| {
                    derived.frame(|derived| {
                        body(&mut GpuFrameRaster {
                            device,
                            queue,
                            renderer: &mut *renderer,
                            resources: &mut *resources,
                            scene: &mut *scene,
                            caches: &mut *caches,
                            images: &mut *images,
                            textures,
                            atlas,
                            derived,
                        })
                    })
                })
            },
            |_, slot| evicted.push(slot.value.id),
        );
        destroy_atlas_images(renderer, resources, device, queue, evicted);
        result
    }
}

impl GpuFrameRaster<'_> {
    /// Draws `scene` into `view` (a texture in [`RASTER_FORMAT`] of the
    /// canvas's `size`, cleared to transparent first) with `encoder`. The
    /// scene is mapped by scale(scale) · translate(−origin).
    ///
    /// The scene is checked whole before anything is recorded. Atlas
    /// uploads are submitted to the queue apart, before the draw in
    /// `encoder` runs.
    pub fn raster(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        scene: &RenderScene,
        canvas: Canvas,
        view: &wgpu::TextureView,
    ) -> Result<(), RenderError> {
        let Canvas {
            origin,
            scale,
            size,
        } = canvas;
        if !scale.is_finite() || scale <= 0.0 {
            return Err(RenderError::InvalidSize);
        }
        validate::target(size)?;
        let max = self.device.limits().max_texture_dimension_2d;
        if u32::from(size.0) > max || u32::from(size.1) > max {
            return Err(RenderError::InvalidSize);
        }
        self.caches.validation.prune();
        validate::scene(scene, &mut self.caches.validation)?;
        let frame = draw::Frame {
            scale,
            origin,
            visible: LogicalRect {
                x: origin.x,
                y: origin.y,
                width: f64::from(size.0) / scale,
                height: f64::from(size.1) / scale,
            },
        };
        self.record(scene, &frame, size);
        self.draw(encoder, size, view)
    }

    /// Draw the recorded scene into `view` with `encoder`.
    fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        (width, height): (u16, u16),
        view: &wgpu::TextureView,
    ) -> Result<(), RenderError> {
        self.renderer
            .render(
                self.scene,
                self.resources,
                self.device,
                self.queue,
                encoder,
                &vello_hybrid::RenderSize {
                    width: u32::from(width),
                    height: u32::from(height),
                },
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

/// Destroys the atlas images of the slots the frame did not draw, in one
/// submission on `queue`. Nothing is submitted when `evicted` is empty.
fn destroy_atlas_images(
    renderer: &mut Renderer,
    resources: &mut Resources,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    evicted: Vec<ImageId>,
) {
    if evicted.is_empty() {
        return;
    }
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Arvane atlas uploads"),
    });
    for id in evicted {
        renderer.destroy_image(resources, &mut encoder, id);
    }
    queue.submit([encoder.finish()]);
}

/// Bound the glyph atlas allocation to the renderer's configured page budget.
fn render_settings() -> RenderSettings {
    RenderSettings {
        memory_settings: MemorySettings {
            image_atlas_config: AtlasConfig {
                atlas_size: (GLYPH_PAGE, GLYPH_PAGE),
                max_atlases: GLYPH_PAGES,
                initial_atlas_count: 0,
                ..AtlasConfig::default()
            },
            ..MemorySettings::default()
        },
        ..RenderSettings::default()
    }
}
