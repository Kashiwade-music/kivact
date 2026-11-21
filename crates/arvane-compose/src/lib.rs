//! Role: Composes a planned frame on the CPU or the GPU, with its exports.
//!
//! The plan (`plan`) knows no backend. The CPU compositor (`cpu`) and the GPU
//! executor (`gpu`, feature `gpu`) draw the same plan into the same images.

mod build;
mod cache;
mod cpu;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(not(feature = "gpu"))]
mod gpu_unbuilt;
mod output;
mod plan;
mod tables;

pub use arvane_protocol::{FrameReport, PixelRect};
pub(crate) use arvane_render::RenderError;
pub use build::{GPU_BUILT, GPU_INTEROP_BUILT};
pub use cpu::{CpuCompositor, CpuOutput, Rendered};
#[cfg(feature = "gpu")]
pub use gpu::{
    GpuContext, GpuDevice, GpuEnv, GpuSurface, Presentation, SurfaceConfigurer, probe_adapter,
};
#[cfg(not(feature = "gpu"))]
pub use gpu_unbuilt::{
    GpuContext, GpuDevice, GpuEnv, GpuSurface, Presentation, SurfaceConfigurer, probe_adapter,
};
pub use kivact_types::export::{ExportFrame, ExportedFrame};
pub use output::GpuReport;
pub use plan::Damage;
