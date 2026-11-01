//! Role: Export pool configuration, frame values and bounded slot bookkeeping.

use crate::common::Error;
use kivact_types::{ExternalFrame, ExternalImageId};
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
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
        ExternalFrame::new(image, self.slot, self.value)
    }
}

/// Which texture the next frame goes into and which values the frames of
/// each texture had: round robin, values counting up from 1.
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    allow(dead_code, reason = "no backend on this platform")
)]
#[derive(Debug)]
pub(crate) struct Slots {
    /// The value of the last frame drawn into each texture (0: none).
    drawn: Vec<u64>,
    /// The texture of the next frame.
    next: usize,
    /// The last value given out (0: none).
    value: u64,
    /// A frame was acquired and not finished.
    pending: bool,
}

#[cfg_attr(
    not(any(windows, target_os = "linux")),
    allow(dead_code, reason = "no backend on this platform")
)]
impl Slots {
    pub(crate) fn new(slots: u32) -> Self {
        Self {
            drawn: vec![0; slots as usize],
            next: 0,
            value: 0,
            pending: false,
        }
    }

    /// The next frame's texture and the value its last frame had; an
    /// error when a frame is acquired and not finished yet.
    pub(crate) fn acquire(&mut self) -> Result<(u32, u64), Error> {
        if self.pending {
            return Err(Error::Mismatch(
                "a frame was acquired and not finished".into(),
            ));
        }
        self.pending = true;
        Ok((self.next as u32, self.drawn[self.next]))
    }

    /// The acquired frame of texture `slot` is submitted: its value.
    pub(crate) fn finish(&mut self, slot: u32) -> FrameReady {
        self.value += 1;
        self.drawn[slot as usize] = self.value;
        self.next = (slot as usize + 1) % self.drawn.len();
        self.pending = false;
        FrameReady {
            slot,
            value: self.value,
        }
    }

    /// The acquired frame is given up without a submission.
    pub(crate) fn cancel(&mut self) {
        self.pending = false;
    }

    /// The value of the last frame drawn into `slot`, when the slot is
    /// not being drawn now.
    pub(crate) fn drawn(&self, slot: u32) -> Result<u64, Error> {
        let Some(&value) = self.drawn.get(slot as usize) else {
            return Err(Error::Mismatch(format!(
                "no texture {slot} in a pool of {}",
                self.drawn.len()
            )));
        };
        if self.pending && slot as usize == self.next {
            return Err(Error::Mismatch(format!("texture {slot} is being drawn")));
        }
        Ok(value)
    }

    /// The last value given out.
    pub(crate) fn last(&self) -> u64 {
        self.value
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
