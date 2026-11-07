//! Role: Native window and input ports.
pub mod embed;
#[cfg(target_os = "linux")]
pub(crate) mod linux_settings;
#[cfg(all(windows, feature = "embed"))]
pub(crate) mod win32;
pub(crate) mod winit;
#[cfg(all(target_os = "linux", feature = "embed"))]
pub(crate) mod x11;
