//! Role: The softbuffer CPU surface of a window and whether it shows a frame.
use arvane_shell::ShellError;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::num::NonZeroU32;

/// A window's CPU presentation surface.
pub(crate) enum CpuSurface<D, W> {
    /// None: not made yet, released for another presenter, or dropped after a failure.
    Released,
    /// A surface whose offers committed no frame yet.
    Ready(softbuffer::Surface<D, W>),
    /// A surface that committed a frame the window shows (a later declined or failed offer does
    /// not change what the window shows).
    Shown(softbuffer::Surface<D, W>),
}

impl<D: HasDisplayHandle, W: HasWindowHandle> CpuSurface<D, W> {
    /// Whether a committed CPU frame is what the window shows.
    pub(crate) fn shows_frame(&self) -> bool {
        matches!(self, Self::Shown(_))
    }

    /// Drop the surface (and its pixel buffer) for another presenter or after a failure.
    pub(crate) fn release(&mut self) {
        *self = Self::Released;
    }

    /// Offer a `width` by `height` buffer to `draw` once and present it when `draw` accepts;
    /// `make` creates the surface when there is none.
    pub(crate) fn present(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
        make: impl FnOnce() -> Result<softbuffer::Surface<D, W>, ShellError>,
        draw: &mut dyn FnMut(&mut [u32]) -> bool,
    ) -> Result<bool, ShellError> {
        let (mut surface, shown) = match std::mem::replace(self, Self::Released) {
            Self::Released => (make()?, false),
            Self::Ready(surface) => (surface, false),
            Self::Shown(surface) => (surface, true),
        };
        let presented = offer(&mut surface, width, height, draw);
        *self = if shown || matches!(presented, Ok(true)) {
            Self::Shown(surface)
        } else {
            Self::Ready(surface)
        };
        presented
    }
}

fn offer<D: HasDisplayHandle, W: HasWindowHandle>(
    surface: &mut softbuffer::Surface<D, W>,
    width: NonZeroU32,
    height: NonZeroU32,
    draw: &mut dyn FnMut(&mut [u32]) -> bool,
) -> Result<bool, ShellError> {
    surface.resize(width, height).map_err(error)?;
    let mut buffer = surface.buffer_mut().map_err(error)?;
    if !draw(&mut buffer[..]) {
        return Ok(false);
    }
    buffer.present().map(|()| true).map_err(error)
}

fn error(error: softbuffer::SoftBufferError) -> ShellError {
    ShellError::new(format!("CPU surface: {error}"))
}
