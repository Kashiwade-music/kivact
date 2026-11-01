//! Role: The test engine on Kivact's wgpu, drawing into a shared pool.
//!
//! Its own `wgpu::Instance` of the window's graphics API and a device of
//! `gpu_share::wgpu::request_device` on the adapter the
//! window reported; the pool (`gpu_share::wgpu::Pool`)
//! makes the shared textures as `wgpu::Texture`s. A frame: the CPU's
//! pixels written into a texture of the engine's own
//! (`Queue::write_texture`), copied into the pool's texture, a render
//! pass on it that keeps the pixels (`LoadOp::Load`, nothing drawn: the
//! texture is a render target too), and the pool's submission.

#![allow(
    unsafe_code,
    reason = "the pool's submission is an unsafe fn: the queue's external synchronization"
)]

use crate::engine::{Engine, POOL, WAIT};
use gpu_share::wgpu::{FORMAT, Pool, request_device};
use gpu_share::{FrameReady, PoolConfig};
use std::time::Duration;
use ui_types::{GpuAdapter, GpuBackend, SharedTextures};

pub(crate) struct Wgpu {
    /// Dropped first: it waits for the last frame.
    pool: Pool,
    staging: wgpu::Texture,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Wgpu {
    /// An engine of a pool of `width` × `height` textures.
    pub(crate) fn new(
        adapter: &GpuAdapter,
        (width, height): (u32, u32),
    ) -> Result<(Self, SharedTextures), String> {
        let backends = match adapter.backend {
            GpuBackend::Dx12 => wgpu::Backends::DX12,
            GpuBackend::Vulkan => wgpu::Backends::VULKAN,
        };
        // As the window's: no Direct3D 12 debug layer turned on after the
        // window's device exists (that removes the device).
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::empty().with_env(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let desc = wgpu::DeviceDescriptor {
            label: Some("gpu-interop-producer"),
            ..Default::default()
        };
        let (_, device, queue) =
            request_device(&instance, adapter, &desc).map_err(|e| e.to_string())?;
        let config = PoolConfig {
            slots: POOL as u32,
            ..PoolConfig::new(width, height)
        };
        let (pool, shared) = Pool::new(&device, adapter, config).map_err(|e| e.to_string())?;
        let staging = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gpu-interop-producer staging"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        Ok((
            Self {
                pool,
                staging,
                device,
                queue,
            },
            shared,
        ))
    }
}

impl Engine for Wgpu {
    fn pitch(&self) -> usize {
        self.staging.width() as usize * 4
    }

    fn draw(&mut self, rgba: &[u8]) -> Result<FrameReady, String> {
        let target = self.pool.acquire(WAIT).map_err(|e| e.to_string())?;
        let size = self.staging.size();
        self.queue.write_texture(
            self.staging.as_image_copy(),
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.width * 4),
                rows_per_image: Some(size.height),
            },
            size,
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_texture_to_texture(
            self.staging.as_image_copy(),
            target.texture().as_image_copy(),
            size,
        );
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        // SAFETY: the engine's own queue of the pool's device, used by this
        // thread alone.
        unsafe { self.pool.submit(&self.queue, target, [encoder.finish()]) }
            .map_err(|e| e.to_string())
    }

    fn release_value(&self) -> u64 {
        self.pool.release_value().unwrap_or(0)
    }

    fn wait_release(&self, value: u64, timeout: Duration) -> bool {
        self.pool.wait_release(value, timeout).is_ok()
    }

    fn read_back(&self, slot: u32) -> Result<Vec<u8>, String> {
        // SAFETY: the engine's own queue, used by this thread alone; the
        // caller's window does not use the texture.
        unsafe { self.pool.read_back(slot, WAIT) }.map_err(|e| e.to_string())
    }
}
