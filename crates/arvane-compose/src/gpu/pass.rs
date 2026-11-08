//! Role: Helpers the GPU processes share: render passes, views and copies.
//!
//! No process module depends on another one: the frame, draw, filter, graph,
//! mask, shader, surface and export processes all take their common helpers
//! from here.

use super::external;
use super::pipelines::buffer;
use crate::ComposeError;
use kivact_types::pixel::BlendMode;
use kivact_types::render::FrameReport;

/// The textures one frame draws with: its targets, by target id, and the external frames it may read.
pub(super) struct FrameTextures<'a> {
    pub(super) targets: Vec<wgpu::Texture>,
    pub(super) external: &'a external::Frame,
}

impl FrameTextures<'_> {
    /// The texture of target `id`.
    pub(super) fn target(&self, id: u32) -> Result<&wgpu::Texture, ComposeError> {
        self.targets
            .get(id as usize)
            .ok_or_else(|| ComposeError::InvalidComposition(format!("target {id} is out of range")))
    }
}

/// One drawn frame: its output (the window's size) and what it produced.
pub(super) struct Drawn {
    pub(super) output: wgpu::Texture,
    pub(super) report: FrameReport,
}

/// A view of all of `texture`.
pub(super) fn view(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// The size of `texture` in px, as floats.
pub(super) fn size_f(texture: &wgpu::Texture) -> (f32, f32) {
    (texture.width() as f32, texture.height() as f32)
}

/// Run `draw` in one render pass into `view`, loaded as `load`.
pub(super) fn with_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    draw: impl FnOnce(&mut wgpu::RenderPass<'_>),
) {
    let attachment = wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load,
            store: wgpu::StoreOp::Store,
        },
    };
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Arvane compositor pass"),
        color_attachments: &[Some(attachment)],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    draw(&mut pass);
}

/// The transparent colour that clears a target.
pub(super) const CLEAR: wgpu::LoadOp<wgpu::Color> = wgpu::LoadOp::Clear(wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
});

/// Unpremultiply RGBA8 in place (straight alpha, as exported pixels are).
pub(super) fn unpremultiply(rgba: &mut [u8]) {
    for px in rgba.as_chunks_mut::<4>().0 {
        let a = u32::from(px[3]);
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
        } else if a < 255 {
            for c in &mut px[..3] {
                *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
}

/// Keeps `buffer` alive for the frame (its bind groups reference it).
pub(super) fn uniform(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> wgpu::Buffer {
    buffer(device, queue, wgpu::BufferUsages::UNIFORM, bytes)
}

/// Copy all of `src` into `dst` (the same size and format).
pub(super) fn copy(encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture, dst: &wgpu::Texture) {
    encoder.copy_texture_to_texture(src.as_image_copy(), dst.as_image_copy(), src.size());
}

/// The σ (px) per axis when the blur changes the image: None when both are below half a pixel.
pub(super) fn blur_sigmas(sigma_x: f64, sigma_y: f64) -> Option<(f64, f64)> {
    (sigma_x >= 0.5 || sigma_y >= 0.5).then_some((sigma_x, sigma_y))
}

/// The index of a blend mode in the draw shader (`blend_fn`); PlusLighter is 255.
pub(super) fn blend_index(mode: BlendMode) -> u32 {
    match mode {
        BlendMode::Normal => 0,
        BlendMode::Multiply => 1,
        BlendMode::Screen => 2,
        BlendMode::Overlay => 3,
        BlendMode::Darken => 4,
        BlendMode::Lighten => 5,
        BlendMode::ColorDodge => 6,
        BlendMode::ColorBurn => 7,
        BlendMode::HardLight => 8,
        BlendMode::SoftLight => 9,
        BlendMode::Difference => 10,
        BlendMode::Exclusion => 11,
        BlendMode::Hue => 12,
        BlendMode::Saturation => 13,
        BlendMode::Color => 14,
        BlendMode::Luminosity => 15,
        BlendMode::PlusLighter => 255,
    }
}
