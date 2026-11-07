//! Role: Offscreen rendering and readback of textures (tests, GPU snapshots, exported pixels).

use super::state::GpuContext;
use crate::ComposeError;
use kivact_types::render::{Composition, CpuFrame};
use std::sync::mpsc;
use std::time::Duration;

impl GpuContext {
    /// Render `composition` into an offscreen frame of `size` px at `scale`, and read it back
    /// as premultiplied RGBA8.
    pub fn render(
        &mut self,
        composition: &Composition,
        size: (u32, u32),
        scale: f64,
    ) -> Result<CpuFrame, ComposeError> {
        let drawn = self.draw_frame(composition, size, scale)?;
        let frame = self.read_frame(&drawn.output);
        self.pool.give(drawn.output);
        frame
    }

    /// Copy `texture` (premultiplied RGBA8) into a frame, waiting a bounded time.
    pub(crate) fn read_frame(&self, texture: &wgpu::Texture) -> Result<CpuFrame, ComposeError> {
        let rgba = self.read_rgba(texture)?;
        Ok(CpuFrame {
            width: texture.width(),
            height: texture.height(),
            rgba,
        })
    }

    /// The premultiplied RGBA8 bytes of `texture`, rows without padding, waiting a bounded time.
    pub(crate) fn read_rgba(&self, texture: &wgpu::Texture) -> Result<Vec<u8>, ComposeError> {
        let (width, height) = (texture.width(), texture.height());
        // wgpu requires rows aligned to COPY_BYTES_PER_ROW_ALIGNMENT during copy.
        let row_bytes = width * 4;
        let stride = row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Arvane readback"),
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
            rgba.extend_from_slice(&row[..row_bytes as usize]);
        }
        drop(data);
        buffer.unmap();
        self.check()?;
        Ok(rgba)
    }
}

/// A backend failure with its message.
pub(crate) fn backend(error: impl std::fmt::Display) -> ComposeError {
    ComposeError::Backend(error.to_string())
}
