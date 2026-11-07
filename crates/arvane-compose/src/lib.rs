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
mod plan;

pub use arvane_render::RenderError as ComposeError;
pub use build::{GPU_BUILT, GPU_INTEROP_BUILT};
pub use cpu::{CpuCompositor, CpuOutput, Rendered};
#[cfg(feature = "gpu")]
pub use gpu::{GpuContext, GpuEnv, GpuSurface, Presentation};
#[cfg(not(feature = "gpu"))]
pub use gpu_unbuilt::{GpuContext, GpuEnv, GpuSurface, Presentation};
pub use kivact_types::render::FrameReport as GpuReport;
pub use kivact_types::render::{ExportFrame, ExportedFrame, FrameReport, PixelRect};
pub use plan::Damage;
