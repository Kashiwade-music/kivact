//! Role: Composite layer trees of one frame into draw operations for the CPU and GPU executors.
//!
//! The plan is backend-independent: it decides layer surfaces, chunk rasters,
//! 3D order, cache reuse and damage, and an executor draws it.

mod cache;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(not(feature = "gpu"))]
mod gpu_unbuilt;
mod plan;

pub use arvane_render::RenderError as ComposeError;
#[cfg(feature = "gpu")]
pub use gpu::{GpuContext, GpuEnv, GpuSurface, Presentation};
#[cfg(not(feature = "gpu"))]
pub use gpu_unbuilt::{GpuContext, GpuEnv, GpuSurface, Presentation};
pub use kivact_types::render::FrameReport;

/// What a presented frame produced besides pixels (design §10.6, `FrameReport`).
pub type GpuReport = FrameReport;

/// Whether this build contains the GPU executor (feature `gpu`).
pub const GPU_BUILT: bool = cfg!(feature = "gpu");
