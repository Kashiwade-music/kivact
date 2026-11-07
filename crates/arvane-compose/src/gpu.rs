//! Role: The GPU executor (feature `gpu`): draws a planned frame with wgpu and presents it.
//!
//! Each target of the plan is a texture, each chunk is rastered by
//! `arvane_render::GpuRaster`, and each operation is one render or copy pass.
//! The device, the pipelines, the window surfaces and the external frames
//! belong to the context; the plan decides everything else.

mod context;
mod draw;
mod exec;
mod export;
mod external;
mod filters;
mod graph;
mod mask;
mod offscreen;
mod pipelines;
mod shader;
mod shared_images;
mod state;
mod surface;
mod targets;

pub use state::{GpuContext, GpuEnv, GpuSurface, Presentation};
