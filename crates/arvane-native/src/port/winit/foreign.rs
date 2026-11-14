//! Role: The native child window that fills a window whose content another
//! library draws.

#[cfg(windows)]
mod win32;
#[cfg(target_os = "linux")]
mod x11;

#[cfg(windows)]
pub(super) use win32::{ForeignConnections, ForeignOs};
#[cfg(target_os = "linux")]
pub(super) use x11::{ForeignConnections, ForeignOs};

#[cfg(not(any(windows, target_os = "linux")))]
mod unsupported;

#[cfg(not(any(windows, target_os = "linux")))]
pub(super) use unsupported::{ForeignConnections, ForeignOs};
