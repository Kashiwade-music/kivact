//! Role: RenderScene presentation and frame policy.
//!
//! Presenting scenes to a window: the renderers, their life cycle and the
//! frame policy behind [`crate::Window::present`]. Each window has one
//! presenter; the host only hands it a scene and learns what happened
//! ([`Presented`]).
//!
//! Frames go through the window's GPU surface when the GPU renderer is
//! ready (feature `gpu`), else they are rasterized on the CPU straight into
//! the window's CPU surface. A native window's presenter starts the GPU on
//! a worker thread with the window (the first frames are drawn on the CPU)
//! and moves the start along on the shell's wakes (`gpu.rs`).
//!
//! Invariants:
//! - A frame the GPU cannot draw because its device failed, or because the
//!   window's surface keeps failing, is drawn on the CPU in the same call,
//!   so neither loses a frame or ends the application.
//! - A frame the window has no buffer for (timed out, occluded) is not
//!   shown: it is to be drawn again a display period later, the wait
//!   doubling while it stays so, up to 16 periods ([`Presented::Skipped`]);
//!   a shown frame or a lost surface starts the wait again at one period.
//! - A scene vello_hybrid cannot draw though the device is fine (an atlas
//!   or intermediate texture it cannot allocate, a missing texture
//!   binding: `RenderError::GpuLimit`) is drawn by vello_cpu for that
//!   frame alone and uploaded through the window's GPU surface (a window
//!   with a swapchain does not show what is presented through GDI).
//! - A failed presentation recreates the window's surface
//!   ([`Presented::SurfaceLost`]); the third in a row is an error (a shown
//!   or skipped frame ends the row), as is a surface that cannot be made
//!   again.
//! - A scene that cannot be drawn is [`Presented::Dropped`]; so is, on
//!   every renderer and before anything is drawn, a frame of a window of
//!   no pixels (minimized, sized 0: `Content`) or without a surface
//!   (destroyed by the platform: `Unavailable`). The resize that gives the
//!   window pixels back asks for a frame, so a dropped one is not retried.
//! - [`Presented::Shown`] means the window accepted the frame, not that it
//!   was scanned out.
//! - Renderer changes are noted to the window's observer as they happen
//!   ([`RendererChange`]: one line starting with `renderer=`), and so is an
//!   external frame the GPU did not draw (not ready within 100 ms, or
//!   already given back to its engine), once per frame value.
//! - Images an external render engine shares (feature `gpu-interop`) are
//!   imported into the window's own device and go with it: a device
//!   failure, a new device or the presenter's drop releases every frame
//!   of them (`arvane-render`'s `gpu/external.rs`).

mod frame;
mod frame_policy;
#[cfg(feature = "gpu")]
mod gpu;
mod health;

pub use frame::{FramePlan, Presented};
pub(crate) use frame::{no_pixels, no_surface};
pub(crate) use frame_policy::Presenter;
