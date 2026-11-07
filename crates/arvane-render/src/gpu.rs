//! Role: GPU rasterizer (feature `gpu`): draws scenes into the textures a caller gives it.
//!
//! Responsibility: one vello_hybrid renderer on the caller's device and
//! queue ([`GpuRaster`]), drawing a scene into a texture view the caller
//! encodes into. Devices, surfaces, external frames and readback belong to
//! the compositor, not here.
//!
//! vello_hybrid is vello_cpu's pipeline with its last stage on the GPU: the
//! CPU flattens paths and makes sparse strips exactly as vello_cpu does
//! (`draw.rs` records the same calls for both), and ordinary render
//! pipelines (no compute shaders) fill the strips' pixels into the target.
//! Its GPU memory follows the scene (strips, a depth texture, glyph atlas
//! pages once text is drawn, a texture per image, layer textures once a
//! layer is used), with no fixed working buffers.
//!
//! Invariants:
//! - GPU resources are serially owned by their rasterizer; a rasterizer
//!   is used on the thread that owns its device.
//! - A target view is [`RASTER_FORMAT`], with render-attachment usage, of
//!   the size the raster is asked for.

mod encode;
mod images;

use crate::draw;
use crate::error::RenderError;
use crate::validate;
use images::Images;
use kivact_types::render::RenderScene;
use kivact_types::{Point, Rect as LogicalRect};
use vello_hybrid::{
    AtlasConfig, MemorySettings, RenderSettings, RenderTargetConfig, Renderer, Resources,
};

/// The format a GPU raster is drawn in (premultiplied RGBA8, unorm).
pub const RASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Glyph atlas pages at most (added as needed, `encode.rs`).
const GLYPH_PAGES: usize = 64;

/// The GPU rasterizer: a vello_hybrid renderer and its resources, drawing
/// on the device a caller gives each call.
pub struct GpuRaster {
    renderer: Renderer,
    resources: Resources,
    /// The scene the frames are recorded into, kept for its allocations.
    scene: vello_hybrid::Scene,
    caches: draw::Caches,
    /// The textures of the last frame's images and the atlas slots of its
    /// patterns.
    images: Images,
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
            images: Images::new(),
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
        raster.raster(
            device,
            queue,
            &mut encoder,
            &RenderScene::default(),
            Point { x: 0.0, y: 0.0 },
            1.0,
            (16, 16),
            &view,
        )?;
        queue.submit([encoder.finish()]);
        Ok(raster)
    }

    /// Release rebuildable validation metadata without releasing imports or leases.
    pub fn release_caches(&mut self) {
        self.caches.validation.clear();
    }

    /// Draws `scene` into `view` (a `size` pixel texture in [`RASTER_FORMAT`],
    /// cleared to transparent first) with `encoder`. `origin` is the
    /// logical point at the view's top left corner and `scale` the device
    /// pixels per logical pixel; the scene is mapped by
    /// scale(scale) · translate(−origin).
    ///
    /// The scene is checked whole before anything is recorded. Atlas
    /// uploads are submitted to `queue` apart, before the draw in
    /// `encoder` runs.
    pub fn raster(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        scene: &RenderScene,
        origin: Point,
        scale: f64,
        size: (u16, u16),
        view: &wgpu::TextureView,
    ) -> Result<(), RenderError> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(RenderError::InvalidSize);
        }
        validate::target(size)?;
        let max = device.limits().max_texture_dimension_2d;
        if u32::from(size.0) > max || u32::from(size.1) > max {
            return Err(RenderError::InvalidSize);
        }
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
        self.record(device, queue, scene, &frame, size);
        self.draw(device, queue, encoder, size, view)
    }

    /// Draw the recorded scene into `view` with `encoder`.
    fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        (width, height): (u16, u16),
        view: &wgpu::TextureView,
    ) -> Result<(), RenderError> {
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                device,
                queue,
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

/// Bound the glyph atlas allocation to the renderer's configured page budget.
fn render_settings() -> RenderSettings {
    RenderSettings {
        memory_settings: MemorySettings {
            image_atlas_config: AtlasConfig {
                atlas_size: (encode::GLYPH_PAGE, encode::GLYPH_PAGE),
                max_atlases: GLYPH_PAGES,
                initial_atlas_count: 0,
                ..AtlasConfig::default()
            },
            ..MemorySettings::default()
        },
        ..RenderSettings::default()
    }
}
