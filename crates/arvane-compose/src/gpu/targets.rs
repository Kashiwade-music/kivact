//! Role: Textures of the frame's targets, chunks and scratch passes, with a pool of free targets.

use arvane_render::RASTER_FORMAT;

/// Free target textures kept for reuse (more are dropped).
const FREE_LIMIT: usize = 32;

/// Free target textures, reused by size.
#[derive(Default)]
pub(crate) struct Pool {
    free: Vec<wgpu::Texture>,
}

impl Pool {
    /// A texture of `size`: a free one when there is, else a new one.
    pub(crate) fn take(&mut self, device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
        match self
            .free
            .iter()
            .position(|t| (t.width(), t.height()) == size)
        {
            Some(index) => self.free.swap_remove(index),
            None => target(device, size),
        }
    }

    /// Return a texture for reuse.
    pub(crate) fn give(&mut self, texture: wgpu::Texture) {
        if self.free.len() < FREE_LIMIT {
            self.free.push(texture);
        }
    }

    /// Drop every free texture.
    pub(crate) fn clear(&mut self) {
        self.free.clear();
    }
}

/// A premultiplied RGBA8 texture of `size` that can be drawn, sampled and copied.
pub(crate) fn target(device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Arvane compositor target"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: RASTER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// A mask accumulator of `size` (one float per pixel).
pub(crate) fn accumulator(device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Arvane compositor accumulator"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: super::pipelines::ACC_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
