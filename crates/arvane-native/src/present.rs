//! Role: The softbuffer CPU surface of a window and whether it shows a frame.
use arvane_shell::{BufferAge, ShellError};
use kivact_types::render::PixelRect;
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
    /// `make` creates the surface when there is none. `damage` selects the device pixels
    /// presented (`None` presents the whole buffer).
    pub(crate) fn present(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
        make: impl FnOnce() -> Result<softbuffer::Surface<D, W>, ShellError>,
        damage: Option<&[PixelRect]>,
        draw: &mut dyn FnMut(&mut [u32], BufferAge) -> bool,
    ) -> Result<bool, ShellError> {
        let (mut surface, shown) = match std::mem::replace(self, Self::Released) {
            Self::Released => (make()?, false),
            Self::Ready(surface) => (surface, false),
            Self::Shown(surface) => (surface, true),
        };
        let presented = offer(&mut surface, width, height, damage, draw);
        *self = if shown || matches!(presented, Ok(true)) {
            Self::Shown(surface)
        } else {
            Self::Ready(surface)
        };
        presented
    }

    /// Show the last committed frame again, for a window system repaint. Win32 softbuffer keeps
    /// one buffer per surface, so presenting it untouched copies the previous frame.
    #[cfg(all(windows, feature = "embed"))]
    pub(crate) fn repaint(&mut self) -> Result<(), ShellError> {
        let Self::Shown(surface) = self else {
            return Ok(());
        };
        surface
            .buffer_mut()
            .map_err(error)?
            .present()
            .map_err(error)
    }
}

fn offer<D: HasDisplayHandle, W: HasWindowHandle>(
    surface: &mut softbuffer::Surface<D, W>,
    width: NonZeroU32,
    height: NonZeroU32,
    damage: Option<&[PixelRect]>,
    draw: &mut dyn FnMut(&mut [u32], BufferAge) -> bool,
) -> Result<bool, ShellError> {
    surface.resize(width, height).map_err(error)?;
    let mut buffer = surface.buffer_mut().map_err(error)?;
    // An age of 1 is the frame presented last time; 0 (never presented) and older ages are unknown.
    let age = if buffer.age() == 1 {
        BufferAge::Previous
    } else {
        BufferAge::Unknown
    };
    if !draw(&mut buffer[..], age) {
        return Ok(false);
    }
    match damage {
        Some(rects) => {
            let rects: Vec<softbuffer::Rect> = rects.iter().filter_map(damage_rect).collect();
            buffer.present_with_damage(&rects)
        }
        None => buffer.present(),
    }
    .map(|()| true)
    .map_err(error)
}

/// The softbuffer rectangle of `rect`; an empty rectangle covers no pixel and is dropped.
fn damage_rect(rect: &PixelRect) -> Option<softbuffer::Rect> {
    Some(softbuffer::Rect {
        x: rect.x,
        y: rect.y,
        width: NonZeroU32::new(rect.width)?,
        height: NonZeroU32::new(rect.height)?,
    })
}

fn error(error: softbuffer::SoftBufferError) -> ShellError {
    ShellError::new(format!("CPU surface: {error}"))
}

/// Construct a softbuffer surface for a retained native target.
pub(crate) fn surface<T: HasDisplayHandle + HasWindowHandle>(
    target: &std::sync::Arc<T>,
) -> Result<softbuffer::Surface<std::sync::Arc<T>, std::sync::Arc<T>>, ShellError> {
    let context = softbuffer::Context::new(target.clone()).map_err(error)?;
    softbuffer::Surface::new(&context, target.clone()).map_err(error)
}
