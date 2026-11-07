//! Role: Filter graphs on the CPU: the evaluator and its primitives (§8.8.4).

mod apply;
mod component_transfer;
mod composite;
mod convolve_matrix;
mod displacement_map;
mod lighting;
mod morphology;
mod tables;
pub(crate) mod turbulence;

pub(crate) use apply::apply;
#[cfg(feature = "gpu")]
pub(crate) use tables::{LINEAR_RGB_TO_SRGB_TABLE, SRGB_TO_LINEAR_RGB_TABLE};
