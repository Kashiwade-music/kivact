//! Role: Composite layer trees of one frame into draw operations for the CPU and GPU executors.
//!
//! The plan is backend-independent: it decides layer surfaces, chunk rasters,
//! 3D order, cache reuse and damage, and an executor draws it.

mod cache;
mod plan;
