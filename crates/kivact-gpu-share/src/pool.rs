//! Role: Export pool configuration, frame values and bounded slot bookkeeping.

use crate::common::Error;
use kivact_types::{ExternalFrame, ExternalImageId};
use std::num::NonZeroU64;
use std::time::Duration;

/// Textures in a pool unless asked otherwise: one the window shows, one
/// the engine draws, one in between.
pub const DEFAULT_SLOTS: u32 = 3;

/// The longest a pool waits on the CPU when it is dropped, for the
/// engine's last submissions to finish (they may wait for a release
/// that never comes; then the pool's objects are leaked).
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    allow(dead_code, reason = "no backend on this platform")
)]
pub(crate) const DROP_WAIT: Duration = Duration::from_secs(5);

/// The size of a pool: `slots` textures of `width` × `height` pixels,
/// made for storage writes or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PoolConfig {
    /// Pixel width (at least 1).
    pub width: u32,
    /// Pixel height (at least 1).
    pub height: u32,
    /// Textures in the pool (at least 1; [`DEFAULT_SLOTS`] by default).
    pub slots: u32,
    /// Whether the engine also writes the textures as storage textures
    /// (false by default): Direct3D 12's `ALLOW_UNORDERED_ACCESS`,
    /// Vulkan's `STORAGE` usage ([`kivact_types::SharedTextures::storage`]).
    /// A Vulkan adapter may not export such images.
    pub storage: bool,
}

impl PoolConfig {
    /// A pool of [`DEFAULT_SLOTS`] textures of `width` × `height` pixels,
    /// without storage.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            slots: DEFAULT_SLOTS,
            storage: false,
        }
    }

    /// Whether the size can be made; why not.
    pub(crate) fn check(&self) -> Result<(), Error> {
        if self.width == 0 || self.height == 0 {
            return Err(Error::Mismatch(format!(
                "an empty pool texture ({} x {})",
                self.width, self.height
            )));
        }
        if self.slots == 0 {
            return Err(Error::Mismatch("a pool of no textures".into()));
        }
        Ok(())
    }
}

/// A frame the engine has submitted: the texture it is in and the value
/// the ready fence reaches once it is drawn. The UI names it in a scene
/// ([`FrameReady::frame`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameReady {
    /// The texture of the pool ([`kivact_types::SharedTextures::textures`]'s
    /// index).
    pub slot: u32,
    /// The ready fence's value that says the frame is drawn.
    pub value: u64,
}

impl FrameReady {
    /// This frame of the image the window imported the pool as.
    pub fn frame(self, image: ExternalImageId) -> ExternalFrame {
        ExternalFrame {
            image,
            slot: self.slot,
            value: self.value,
        }
    }
}

/// The texture the next frame is drawn into, as [`Slots::next`] gives it:
/// what a pool's target holds.
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    allow(dead_code, reason = "no backend on this platform")
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Acquired {
    /// The texture's index in the pool.
    pub(crate) slot: u32,
    /// The value of the texture's last frame: the window's `release`
    /// reaches it before the texture is written again (`None`: the texture
    /// has no frame yet).
    pub(crate) wait: Option<NonZeroU64>,
}

/// Which texture the next frame goes into and which values the frames of
/// each texture had: round robin, values counting up from 1. A pool's
/// target borrows the pool until it is submitted or dropped, so one frame
/// is drawn at a time and the frame finished is the one acquired.
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    allow(dead_code, reason = "no backend on this platform")
)]
#[derive(Debug)]
pub(crate) struct Slots {
    /// The value of the last frame drawn into each texture.
    drawn: Vec<Option<NonZeroU64>>,
    /// The texture of the next frame.
    next: usize,
    /// The last value given out.
    last: Option<NonZeroU64>,
}

#[cfg_attr(
    not(any(windows, target_os = "linux")),
    allow(dead_code, reason = "no backend on this platform")
)]
impl Slots {
    pub(crate) fn new(slots: u32) -> Self {
        Self {
            drawn: vec![None; slots as usize],
            next: 0,
            last: None,
        }
    }

    /// The next frame's texture and its last frame's value.
    pub(crate) fn next(&self) -> Acquired {
        Acquired {
            slot: self.next as u32,
            wait: self.drawn[self.next],
        }
    }

    /// The value the next finished frame gets.
    pub(crate) fn next_value(&self) -> NonZeroU64 {
        match self.last {
            Some(last) => last
                .checked_add(1)
                .expect("invariant: frame values fit u64"),
            None => NonZeroU64::MIN,
        }
    }

    /// The frame `acquired` (the one [`Slots::next`] gave) was submitted:
    /// its value.
    pub(crate) fn finish(&mut self, acquired: Acquired) -> FrameReady {
        assert_eq!(
            acquired.slot as usize, self.next,
            "invariant: the frame finished is the one `next` gave (a target borrows its pool)"
        );
        let value = self.next_value();
        self.last = Some(value);
        self.drawn[self.next] = Some(value);
        self.next = (self.next + 1) % self.drawn.len();
        FrameReady {
            slot: acquired.slot,
            value: value.get(),
        }
    }

    /// The value of the last frame drawn into `slot`.
    pub(crate) fn drawn(&self, slot: u32) -> Result<Option<NonZeroU64>, Error> {
        self.drawn.get(slot as usize).copied().ok_or_else(|| {
            Error::Mismatch(format!(
                "no texture {slot} in a pool of {}",
                self.drawn.len()
            ))
        })
    }

    /// The last value given out.
    pub(crate) fn last(&self) -> Option<NonZeroU64> {
        self.last
    }
}

/// Premultiplied RGBA8 pixels made straight in place (a pixel of alpha 0
/// becomes transparent black).
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    allow(dead_code, reason = "no backend on this platform")
)]
pub(crate) fn straighten(rgba: &mut [u8]) {
    for pixel in rgba.as_chunks_mut::<4>().0 {
        let alpha = u32::from(pixel[3]);
        match alpha {
            255 => {}
            0 => pixel[..3].fill(0),
            _ => {
                for channel in &mut pixel[..3] {
                    let straight = (u32::from(*channel) * 255 + alpha / 2) / alpha;
                    *channel = straight.min(255) as u8;
                }
            }
        }
    }
}
