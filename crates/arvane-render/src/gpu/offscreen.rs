//! Role: Renders scenes into offscreen textures and reads frames back.

use super::external::ExternalPass;
use super::state::{GpuContext, Offscreen};
use super::texture::target_texture;
use crate::error::RenderError;
use kivact_types::Size;
use kivact_types::render::{CpuFrame, RenderScene};
use std::sync::mpsc;
use std::time::Duration;

impl GpuContext {
    /// Renders a validated scene offscreen into premultiplied RGBA8 and
    /// reads it back (tests and screenshots; windows use
    /// [`GpuContext::present`]).
    pub fn render(
        &mut self,
        scene: &RenderScene,
        size: Size,
        scale: f64,
    ) -> Result<CpuFrame, RenderError> {
        let mut external = std::mem::take(&mut self.external);
        let drawn = (|| {
            let pass = ExternalPass::begin(&mut external, scene)?;
            self.render_frame(scene, size, scale, pass)
        })();
        self.external = external;
        drawn
    }

    /// [`GpuContext::render`] between the external frames' hand-offs.
    fn render_frame(
        &mut self,
        scene: &RenderScene,
        size: Size,
        scale: f64,
        pass: ExternalPass<'_>,
    ) -> Result<CpuFrame, RenderError> {
        let ((width, height), info) = self.dimensions(scene, size, scale)?;
        if !matches!(&self.offscreen, Offscreen::Allocated { size, .. } if *size == (width, height))
        {
            self.offscreen = Offscreen::Allocated {
                texture: target_texture(&self.device, width, height),
                size: (width, height),
            };
        }
        let texture = match &self.offscreen {
            Offscreen::Allocated { texture, .. } => texture.clone(),
            Offscreen::None => unreachable!("invariant: the offscreen texture was just made"),
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane offscreen"),
            });
        self.encode(scene, scale, (width, height), &info, false, pass.frame());
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let drawn = self.draw(&mut encoder, (width, height), &view);
        self.queue.submit([encoder.finish()]);
        let submitted = pass.submit();
        drawn?;
        let drawn = self.read_texture(&texture);
        let ended = submitted.finish();
        let frame = drawn?;
        ended?;
        Ok(frame)
    }

    /// Copy `texture` (in [`FORMAT`]) into an RGBA frame, waiting a
    /// bounded time.
    pub(crate) fn read_texture(&self, texture: &wgpu::Texture) -> Result<CpuFrame, RenderError> {
        let (width, height) = (texture.width(), texture.height());
        // wgpu requires rows aligned to COPY_BYTES_PER_ROW_ALIGNMENT during copy.
        let row_bytes = width * 4;
        let stride = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Arvane raster readback"),
            size: u64::from(stride) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane readback"),
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );
        let submission = self.queue.submit([encoder.finish()]);
        let slice = buffer.slice(..);
        let (sender, receiver) = mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(backend)?;
        receiver
            .recv_timeout(Duration::from_secs(1))
            .map_err(backend)?
            .map_err(backend)?;
        let data = slice.get_mapped_range();
        let mut rgba = Vec::with_capacity((row_bytes * height) as usize);
        for row in data.chunks_exact(stride as usize) {
            // BGRA to RGBA.
            for [b, g, r, a] in row[..row_bytes as usize].as_chunks::<4>().0 {
                rgba.extend_from_slice(&[*r, *g, *b, *a]);
            }
        }
        drop(data);
        buffer.unmap();
        self.check()?;
        Ok(CpuFrame {
            width,
            height,
            rgba,
        })
    }
}

pub(crate) fn backend(error: impl std::fmt::Display) -> RenderError {
    RenderError::Backend(error.to_string())
}
