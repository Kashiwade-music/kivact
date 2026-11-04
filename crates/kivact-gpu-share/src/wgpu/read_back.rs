//! Role: Reads a wgpu texture back to the CPU as straight RGBA8, with wgpu alone.
//!
//! For a window that cannot import (it draws on the CPU, or its GPU is not
//! up yet): the UI shows the frame as `Pixels`. The engine draws into a
//! texture of its own (not shared) and reads it back here.

use crate::common::Error;
use kivact_types::color::unpremultiply;
use std::time::Duration;

/// `texture`'s pixels (premultiplied alpha) as straight-alpha RGBA8
/// (`width` × `height` × 4 bytes, rows top to bottom), copied on `queue`
/// after the earlier submissions and waited for at most `timeout`
/// (`Device::poll`). The texture is a 2D texture of `device` of
/// `Rgba8Unorm` (the pools' format), 1 mip level, 1 layer, 1 sample, with
/// `COPY_SRC` usage, in wgpu's hands (a pool's texture between frames is
/// the window's: read it with the pool's `read_back`).
pub fn read_back(
    device: &::wgpu::Device,
    queue: &::wgpu::Queue,
    texture: &::wgpu::Texture,
    timeout: Duration,
) -> Result<Vec<u8>, Error> {
    if texture.format() != ::wgpu::TextureFormat::Rgba8Unorm
        || texture.dimension() != ::wgpu::TextureDimension::D2
        || texture.mip_level_count() != 1
        || texture.depth_or_array_layers() != 1
        || texture.sample_count() != 1
        || !texture.usage().contains(::wgpu::TextureUsages::COPY_SRC)
    {
        return Err(Error::Mismatch(
            "not a 2D RGBA8 texture of one mip level, layer and sample, with COPY_SRC".into(),
        ));
    }
    let (width, height) = (texture.width(), texture.height());
    let row = width as usize * 4;
    let padded = row.next_multiple_of(::wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize);
    let buffer = device.create_buffer(&::wgpu::BufferDescriptor {
        label: Some("kivact read back"),
        size: (padded * height as usize) as u64,
        usage: ::wgpu::BufferUsages::COPY_DST | ::wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&::wgpu::CommandEncoderDescriptor {
        label: Some("kivact read back"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        ::wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: ::wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded as u32),
                rows_per_image: Some(height),
            },
        },
        texture.size(),
    );
    let index = queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(::wgpu::MapMode::Read, move |mapped| {
            let _ = sender.send(mapped);
        });
    device
        .poll(::wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: Some(timeout),
        })
        .map_err(|error| match error {
            ::wgpu::PollError::Timeout => Error::Timeout("the read back"),
            error => Error::Api {
                call: "Device::poll",
                message: error.to_string(),
            },
        })?;
    match receiver.try_recv() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            return Err(Error::Api {
                call: "Buffer::map_async",
                message: error.to_string(),
            });
        }
        Err(_) => return Err(Error::Timeout("the read back's mapping")),
    }
    let mut rgba = Vec::with_capacity(row * height as usize);
    {
        let mapped = buffer.slice(..).get_mapped_range();
        for line in mapped.chunks(padded) {
            rgba.extend_from_slice(&line[..row]);
        }
    }
    buffer.unmap();
    unpremultiply(&mut rgba);
    Ok(rgba)
}
