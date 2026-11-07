mod cache;
mod cpu;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(not(feature = "gpu"))]
mod gpu_unbuilt;
mod plan;

pub use arvane_render::RenderError as ComposeError;
pub use cpu::{CpuCompositor, CpuOutput, Rendered};
#[cfg(feature = "gpu")]
pub use gpu::{GpuContext, GpuEnv, GpuSurface, Presentation};
#[cfg(not(feature = "gpu"))]
pub use gpu_unbuilt::{GpuContext, GpuEnv, GpuSurface, Presentation};
pub use kivact_types::render::{ExportFrame, ExportedFrame, FrameReport, PixelRect};
pub use plan::Damage;

/// What a presented frame produced besides pixels (design §10.6, `FrameReport`).
pub type GpuReport = FrameReport;

/// Whether this build contains the GPU executor (feature `gpu`).
pub const GPU_BUILT: bool = cfg!(feature = "gpu");
