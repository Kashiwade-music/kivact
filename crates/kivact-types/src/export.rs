//! Role: Exported drawable frames and frame statistics shared by the engine, Arvane and core.
use crate::ids::DrawableId;
use crate::pixel::Pixels;

/// An exported drawable frame; the generation travels in ExportedFrame.
#[derive(Clone, Debug, PartialEq)]
pub enum ExportFrame {
    /// The frame as pixels.
    Pixels(Pixels),
    /// The frame as a slot of a shared pool, released when the fence reaches `value`.
    Gpu {
        /// The pool slot.
        slot: u32,
        /// The fence value that signals the frame is ready.
        value: u64,
    },
}

/// One exported frame of the request `generation` of `sink`, from the
/// compositor through arvane and the engine to core unchanged.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportedFrame {
    /// The drawable that receives the frame.
    pub sink: DrawableId,
    /// The request this answers.
    pub generation: u64,
    /// The frame itself.
    pub frame: ExportFrame,
}

/// Work a frame did (statistics, S-9), for diagnostics and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FrameStats {
    /// Chunks rastered by Vello.
    pub chunks_rastered: u32,
    /// Pixels rastered.
    pub pixels_rastered: u64,
    /// Surfaces reused from the cache.
    pub surfaces_reused: u32,
    /// Area of the damage.
    pub damage_pixels: u64,
    /// Targets used by the frame.
    pub targets: u32,
}
