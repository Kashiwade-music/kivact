//! Role: Composite layer trees of one frame into draw operations for the CPU and GPU executors.
//!
//! The plan is backend-independent: it decides layer surfaces, chunk rasters,
//! 3D order, cache reuse and damage, and an executor draws it.

mod cache;
mod cpu;
mod plan;

pub use arvane_render::RenderError as ComposeError;
pub use cpu::{CpuCompositor, CpuOutput, Rendered};
pub use kivact_types::render::{ExportFrame, ExportedFrame, FrameReport, PixelRect};
pub use plan::Damage;
