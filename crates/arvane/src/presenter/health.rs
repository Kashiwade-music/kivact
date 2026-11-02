//! Role: What a window's surface did with the last frames that reached it, for the frame policy.

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
    /// The last `losses` frames lost the surface, which was made again.
    Losing { losses: NonZeroU8 },
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

    /// A frame lost the surface: whether it is the third loss in a row
    /// (presenting fails then).
    pub(super) fn lost(&mut self) -> bool {
        let losses = match *self {
            Self::Losing { losses } => losses.saturating_add(1),
            Self::Healthy | Self::Skipping { .. } => NonZeroU8::MIN,
        };
        *self = Self::Losing { losses };
        losses.get() >= LOSSES
    }
}
