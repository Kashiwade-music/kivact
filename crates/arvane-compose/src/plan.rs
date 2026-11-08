//! Role: Plans one composition frame into draw operations, with cache reuse and damage.
//!
//! The plan knows no backend. A CPU or GPU executor draws its [`Op`]s in
//! order into the targets it allocates from [`Plan::targets`], and rasters
//! the [`ChunkTask`]s first. Records are returned, not stored: the executor
//! stores them after a successful frame.

mod bounds;
mod builder;
mod context3d;
mod damage;
mod frame;
mod hash;
mod items;
mod layers;
mod ops;
mod polygon;
mod validate;

pub(crate) use builder::build;
pub use ops::Damage;
#[cfg(feature = "gpu")]
pub(crate) use ops::INLINE_TARGET_LIMIT;
pub(crate) use ops::{
    ChunkTask, DrawOp, Op, PlannedExport, Redraw, Sampling, Scissor, Source, TargetKind, TargetSpec,
};
#[cfg(feature = "gpu")]
pub(crate) use ops::{Plan, ShaderOp};
