//! Role: The GPU export pools of drawables (design §8.12): a frame is copied into a free slot of its sink's pool and shared, and a frame that finds no free slot waits for the next one.

use std::collections::HashMap;

use super::state::GpuContext;
use crate::ComposeError;
use crate::plan::PlannedExport;
use kivact_types::ids::DrawableId;
use kivact_types::render::{Composition, Export, ExportFrame, ExportMode, ExportedFrame};
use kivact_types::{GpuAdapter, SharedTextures};

/// The export pool of a sink (feature `gpu-interop`).
#[cfg(feature = "gpu-interop")]
pub(crate) type Pool = kivact_gpu_share::wgpu::Pool;

/// Without the `gpu-interop` feature no pool is ever made.
#[cfg(not(feature = "gpu-interop"))]
pub(crate) enum Pool {}

/// A sink's export pool with the request it serves.
pub(crate) struct ExportPool {
    /// The request the pool serves.
    generation: u64,
    /// The pool's texture size in px.
    size: (u32, u32),
    /// The pool itself.
    pool: Pool,
}

/// What a sink last received, by which the compositor decides whether to send another frame (design §8.12).
#[derive(Clone, Copy, Debug, PartialEq)]
struct ExportRecord {
    /// The request answered.
    generation: u64,
    /// How the frame was delivered.
    mode: ExportMode,
    /// The layer's content signature.
    signature: u64,
    /// Size in device px.
    size: (u32, u32),
    /// Device px per logical px.
    scale: f64,
}

/// The export state of a GPU executor: its pools by sink, and what each sink last received.
#[derive(Default)]
pub(crate) struct Exports {
    /// The pools by sink.
    pools: HashMap<DrawableId, ExportPool>,
    /// The last record of each sink.
    records: HashMap<DrawableId, ExportRecord>,
}

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
                    super::exec::unpremultiply(&mut rgba);
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
    // STOPPED (design A-2): the hand-off submission is the pool's `unsafe` `Target::submit`, and
    // `unsafe_code` is denied in this crate. Until the orchestrator decides where that boundary sits,
    // a GPU frame never reaches a slot: it stays pending. The slot is not acquired here.
    let _ = (pool, device, queue, texture, size);
    Ok(None)
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
