//! Role: Filter graphs on the CPU: the evaluator and its primitives.

mod apply;
mod component_transfer;
mod composite;
mod convolve_matrix;
mod displacement_map;
mod lighting;
mod morphology;
mod turbulence;

pub(crate) use apply::apply;
