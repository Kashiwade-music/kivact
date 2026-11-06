//! Role: What a window's surface did with the frames that reached it.

use crate::presentation::Renderer;

use std::num::{NonZeroU8, NonZeroU32};

/// Surface failures in a row after which presenting is an error.
const LOSSES: u8 = 3;

/// A skipped frame is drawn again after at most 2^4 = 16 display periods.
const MAX_BACKOFF: u32 = 4;

/// What the window's surface did with the last frames that reached it. A
/// frame dropped before it reached the surface (its scene or size cannot be
/// drawn) leaves it as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FrameHealth {
    /// The last frame was shown, or none reached the surface yet (a new
    /// surface).
    Healthy,
    /// The last `frames` frames had no buffer (hidden or occluded window).
    Skipping { frames: NonZeroU32 },
    /// The last `losses` frames lost the surface, which was made again; the
    /// last `gpu` of them were drawn on the GPU.
    Losing { losses: NonZeroU8, gpu: u8 },
}

pub(super) enum LossAction {
    Recover,
    FallbackToCpu,
    Fail,
}

impl FrameHealth {
    /// A frame was shown.
    pub(super) fn shown(&mut self) {
        *self = Self::Healthy;
    }

    /// A frame had no buffer: how long to wait before drawing it again, one
    /// display period (`period_ns`) doubling with each frame skipped in a
    /// row, up to 16 periods.
    pub(super) fn skipped(&mut self, period_ns: u64) -> u64 {
        let frames = match *self {
            Self::Skipping { frames } => frames.saturating_add(1),
            Self::Healthy | Self::Losing { .. } => NonZeroU32::MIN,
        };
        *self = Self::Skipping { frames };
        period_ns << (frames.get() - 1).min(MAX_BACKOFF)
    }

    /// Count a consecutive surface loss and choose recovery, CPU fallback,
    /// or failure according to the renderer that lost it: losses on any
    /// renderer count toward failure, only consecutive GPU losses toward the
    /// CPU fallback.
    pub(super) fn lost(&mut self, renderer: Renderer) -> LossAction {
        let (losses, gpu) = match *self {
            Self::Losing { losses, gpu } => (losses.saturating_add(1), gpu),
            Self::Healthy | Self::Skipping { .. } => (NonZeroU8::MIN, 0),
        };
        let gpu = if renderer == Renderer::Gpu {
            gpu.saturating_add(1)
        } else {
            0
        };
        *self = Self::Losing { losses, gpu };
        if losses.get() >= LOSSES {
            LossAction::Fail
        } else if gpu == 2 {
            LossAction::FallbackToCpu
        } else {
            LossAction::Recover
        }
    }
}
