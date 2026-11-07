//! Role: The frame report alias of the GPU executor.

use kivact_types::render::FrameReport;

/// What a presented frame produced besides pixels (design §10.6, `FrameReport`).
pub type GpuReport = FrameReport;
