//! Role: The CPU executor: draws planned frames into premultiplied RGBA8 pixmaps.

mod blend;
mod blur;
mod compositor;
mod draw;
mod effects;
mod filters;
pub(crate) mod graph;
mod mask;
mod surface;

pub use compositor::{CpuCompositor, CpuOutput, Rendered};
