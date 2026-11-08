//! Role: The export pools by sink, and what each sink last received.
//!
//! Passive state of the GPU context: the processes that make and use the
//! pools live in `export`.

use std::collections::HashMap;

use kivact_types::ids::DrawableId;
use kivact_types::render::ExportMode;

/// The export pool of a sink (feature `gpu-interop`).
#[cfg(feature = "gpu-interop")]
pub(crate) type Pool = kivact_gpu_share::wgpu::Pool;

/// Without the `gpu-interop` feature no pool is ever made.
#[cfg(not(feature = "gpu-interop"))]
pub(crate) enum Pool {}

/// A sink's export pool with the request it serves.
pub(super) struct ExportPool {
    /// The request the pool serves.
    pub(super) generation: u64,
    /// The pool's texture size in px.
    pub(super) size: (u32, u32),
    /// The pool itself.
    pub(super) pool: Pool,
}

/// What a sink last received, by which the compositor decides whether to send another frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ExportRecord {
    /// The request answered.
    pub(super) generation: u64,
    /// How the frame was delivered.
    pub(super) mode: ExportMode,
    /// The layer's content signature.
    pub(super) signature: u64,
    /// Size in device px.
    pub(super) size: (u32, u32),
    /// Device px per logical px.
    pub(super) scale: f64,
}

/// The export state of a GPU executor: its pools by sink, and what each sink last received.
#[derive(Default)]
pub(crate) struct Exports {
    /// The pools by sink.
    pub(super) pools: HashMap<DrawableId, ExportPool>,
    /// The last record of each sink.
    pub(super) records: HashMap<DrawableId, ExportRecord>,
}
