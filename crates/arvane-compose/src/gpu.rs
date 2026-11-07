//! Role: The GPU executor (feature `gpu`): declarations and re-exports only.

mod context;
mod draw;
mod exec;
mod export;
mod export_pool;
mod external;
mod filters;
mod graph;
mod mask;
mod offscreen;
mod pass;
mod pipelines;
mod shader;
mod shader_cache;
mod shared_images;
mod state;
mod surface;
mod targets;

pub use state::{GpuContext, GpuEnv, GpuSurface, Presentation};
