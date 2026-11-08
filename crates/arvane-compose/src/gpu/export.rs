//! Role: Sends drawable frames to their sinks' export pools (design §8.12).
//!
//! A frame is copied into a free slot of its sink's pool and shared; a frame
//! that finds no free slot waits for the next one. The pools and the records
//! of what each sink received are passive state in `export_pool`.

use super::export_pool::{ExportPool, ExportRecord, Pool};
use super::pass::unpremultiply;
use super::state::GpuContext;
use crate::ComposeError;
use crate::plan::PlannedExport;
use kivact_types::ids::DrawableId;
use kivact_types::render::{Composition, Export, ExportFrame, ExportMode, ExportedFrame};
use kivact_types::{GpuAdapter, SharedTextures};

impl GpuContext {
    /// The export pool of `sink` for request `generation` on `adapter`, as shared textures (`size` device px).
    /// A pool the sink had before is dropped first.
    pub fn export_drawable(
        &mut self,
        sink: DrawableId,
        generation: u64,
        adapter: &GpuAdapter,
        size: (u32, u32),
    ) -> Result<SharedTextures, ComposeError> {
        self.drop_export(sink);
        let (pool, shared) = make_pool(&self.device, adapter, size)?;
        self.exports.pools.insert(
            sink,
            ExportPool {
                generation,
                size,
                pool,
            },
        );
        Ok(shared)
    }

    /// Drop the export pool of `sink`: it leaves the table at once, and is dropped on a worker thread once its last submissions finish.
    pub fn drop_export(&mut self, sink: DrawableId) {
        self.exports.records.remove(&sink);
        if let Some(export) = self.exports.pools.remove(&sink) {
            retire(export.pool);
        }
    }

    /// The exports of a frame (design §8.12): pixel frames read back, GPU frames copied into their sink's pool.
    /// Returns the frames sent and the sinks whose GPU frame waits for a free slot.
    pub(super) fn send_exports(
        &mut self,
        exports: &[PlannedExport],
        composition: &Composition,
        textures: &[wgpu::Texture],
    ) -> Result<(Vec<ExportedFrame>, Vec<DrawableId>), ComposeError> {
        let mut frames = Vec::new();
        let mut pending = Vec::new();
        for (export, planned) in composition.exports.iter().zip(exports) {
            let record = ExportRecord {
                generation: export.generation,
                mode: export.mode,
                signature: planned.signature,
                size: export.size,
                scale: export.scale,
            };
            if self.exports.records.get(&export.sink) == Some(&record) {
                continue;
            }
            let texture = textures.get(planned.target as usize).ok_or_else(|| {
                ComposeError::InvalidComposition("an export target is missing".into())
            })?;
            let frame = match export.mode {
                ExportMode::Pixels => {
                    let mut rgba = self.read_rgba(texture)?;
                    unpremultiply(&mut rgba);
                    let pixels =
                        kivact_types::pixel::Pixels::new(texture.width(), texture.height(), rgba);
                    Some(ExportFrame::Pixels(pixels))
                }
                ExportMode::Gpu => self
                    .send_slot(export, texture)?
                    .map(|(slot, value)| ExportFrame::Gpu { slot, value }),
            };
            match frame {
                Some(frame) => {
                    self.exports.records.insert(export.sink, record);
                    frames.push(ExportedFrame {
                        sink: export.sink,
                        generation: export.generation,
                        frame,
                    });
                }
                None => pending.push(export.sink),
            }
        }
        self.exports
            .records
            .retain(|sink, _| composition.exports.iter().any(|e| e.sink == *sink));
        Ok((frames, pending))
    }

    /// Copy `texture` into a free slot of the sink's pool and publish it: the slot and its ready value,
    /// or None when the sink has no pool for this request or the window has released no slot yet.
    fn send_slot(
        &mut self,
        export: &Export,
        texture: &wgpu::Texture,
    ) -> Result<Option<(u32, u64)>, ComposeError> {
        let Some(pool) = self.exports.pools.get_mut(&export.sink) else {
            return Ok(None);
        };
        if pool.generation != export.generation || pool.size != export.size {
            return Ok(None);
        }
        publish(
            &mut pool.pool,
            &self.device,
            &self.queue,
            texture,
            export.size,
        )
    }
}

/// Make the export pool of `size` on `device` (feature `gpu-interop`), and the handles the window imports.
#[cfg(feature = "gpu-interop")]
fn make_pool(
    device: &wgpu::Device,
    adapter: &GpuAdapter,
    size: (u32, u32),
) -> Result<(Pool, SharedTextures), ComposeError> {
    let config = kivact_gpu_share::PoolConfig::new(size.0, size.1);
    kivact_gpu_share::wgpu::Pool::new(device, adapter, config)
        .map_err(|error| ComposeError::Backend(error.to_string()))
}

/// Without `gpu-interop` there is no pool to make.
#[cfg(not(feature = "gpu-interop"))]
fn make_pool(
    _device: &wgpu::Device,
    _adapter: &GpuAdapter,
    _size: (u32, u32),
) -> Result<(Pool, SharedTextures), ComposeError> {
    Err(ComposeError::Backend(
        "drawable export pools need the gpu-interop feature".into(),
    ))
}

/// Copy `texture` into the next free slot of `pool` and publish the copy (feature `gpu-interop`).
/// None when the window has not released a slot yet; nothing is queued then.
#[cfg(feature = "gpu-interop")]
fn publish(
    pool: &mut Pool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Result<Option<(u32, u64)>, ComposeError> {
    if (texture.width(), texture.height()) != size {
        return Err(ComposeError::InvalidComposition(
            "an export target does not match its export size".into(),
        ));
    }
    // Never waits on the window: a slot the window has not released stays pending.
    let target = pool
        .try_acquire()
        .map_err(|error| ComposeError::Backend(error.to_string()))?;
    let Some(target) = target else {
        return Ok(None);
    };
    let commands = copy_commands(device, texture, target.texture(), size);
    let ready = hand_off(target, queue, commands)?;
    Ok(Some((ready.slot, ready.value)))
}

/// The command buffer copying `source` into `destination`: both are `size` px and `Rgba8Unorm` (`RASTER_FORMAT` is the pool's `FORMAT`), so the copy is valid.
#[cfg(feature = "gpu-interop")]
fn copy_commands(
    device: &wgpu::Device,
    source: &wgpu::Texture,
    destination: &wgpu::Texture,
    size: (u32, u32),
) -> wgpu::CommandBuffer {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Arvane export hand-off copy"),
    });
    encoder.copy_texture_to_texture(
        source.as_image_copy(),
        destination.as_image_copy(),
        wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
    );
    encoder.finish()
}

/// Submit `commands` (the copy into `target`'s slot) as the frame's hand-off to the window: the only `unsafe` of the export path.
#[cfg(feature = "gpu-interop")]
#[allow(
    unsafe_code,
    reason = "publishing a frame into a drawable export pool slot calls kivact-gpu-share's unsafe Target::submit, which submits raw on wgpu's queue"
)]
fn hand_off(
    target: kivact_gpu_share::wgpu::Target<'_>,
    queue: &wgpu::Queue,
    commands: wgpu::CommandBuffer,
) -> Result<kivact_gpu_share::FrameReady, ComposeError> {
    // SAFETY: the contract of `Target::submit` is: "`queue` is the queue of the pool's device. No other thread
    // uses that queue during the call (`Queue::submit`, `Surface::present`, `Device::poll` and the like)."
    // The first part holds because the pool is made on `self.device` (`export_drawable`) and `self.queue` comes from the same
    // `open_device` call as that device (`GpuContext::with_backends`). The second part holds because a `GpuContext` is owned by one
    // thread at a time (it moves between threads by value and is never shared), so the queue is used only by that thread, and the
    // retiring thread of `retire` drops its pool without any queue submission.
    unsafe { target.submit(queue, std::iter::once(commands)) }
        .map_err(|error| ComposeError::Backend(error.to_string()))
}

/// Without `gpu-interop` no pool exists, so nothing is published.
#[cfg(not(feature = "gpu-interop"))]
fn publish(
    _pool: &mut Pool,
    _device: &wgpu::Device,
    _queue: &wgpu::Queue,
    _texture: &wgpu::Texture,
    _size: (u32, u32),
) -> Result<Option<(u32, u64)>, ComposeError> {
    Ok(None)
}

/// Drop `pool` on a worker thread: its drop waits (at most five seconds) for its last submissions (design §8.12).
#[cfg(feature = "gpu-interop")]
fn retire(pool: Pool) {
    std::thread::spawn(move || drop(pool));
}

/// Without `gpu-interop` there is no pool to retire.
#[cfg(not(feature = "gpu-interop"))]
fn retire(pool: Pool) {
    match pool {}
}
