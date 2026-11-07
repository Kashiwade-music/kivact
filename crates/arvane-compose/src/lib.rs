//! Role: Composite layer trees on the CPU and the GPU with caches and damage.

mod cache;
mod cpu;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(not(feature = "gpu"))]
mod gpu_unbuilt;
mod output;
mod plan;
mod tables;

pub use arvane_render::RenderError as ComposeError;
pub use cpu::{CpuCompositor, CpuOutput, Rendered};
#[cfg(feature = "gpu")]
pub use gpu::{GpuContext, GpuEnv, GpuSurface, Presentation};
#[cfg(not(feature = "gpu"))]
pub use gpu_unbuilt::{GpuContext, GpuEnv, GpuSurface, Presentation};
pub use kivact_types::render::{ExportFrame, ExportedFrame, FrameReport, PixelRect};
pub use output::{GPU_BUILT, GpuReport};
pub use plan::Damage;
