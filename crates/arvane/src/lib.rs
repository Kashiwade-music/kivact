//! Role: Arvane's window driver: the shell's handler and every window's presenter.
//!
//! Responsibility: drives a shell backend (`arvane-shell`'s SPI,
//! implemented by `arvane-native` and `arvane-virtual`) and presents
//! each window's composition through `arvane-compose`, owning the
//! renderers' life cycle and the frame policy. It re-exports no other
//! crate: hosts use `arvane-shell`, `arvane-text`, `arvane-asset`,
//! `arvane-render` and `arvane-compose` directly.
#![forbid(unsafe_code)]

mod env;
mod presentation;
mod presenter;
#[cfg(feature = "test")]
mod test_hooks;
mod window;

pub use presentation::{GpuStartCause, Observer, Renderer, RendererChange, observer};
pub use presenter::{ExportNotice, FramePlan, Presented};
#[cfg(feature = "test")]
pub use test_hooks::{gpu_unavailable, lose_gpu_device, test_font};
pub use window::{
    DriverOptions, Handler, SteppedShell, Window, Windows, embed_driver, run, start_stepped,
};
