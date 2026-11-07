//! Role: The frame report alias and the build's GPU feature flag.

use kivact_types::render::FrameReport;

/// What a presented frame produced besides pixels (design §10.6, `FrameReport`).
pub type GpuReport = FrameReport;

/// Whether this build contains the GPU executor (feature `gpu`).
pub const GPU_BUILT: bool = cfg!(feature = "gpu");
