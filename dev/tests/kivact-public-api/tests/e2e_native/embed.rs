//! Role: Shared embedding contracts and platform-specific real-window checks.
#![cfg(any(windows, target_os = "linux"))]
mod contract;
mod panic_boundary;
mod support;
#[cfg(windows)]
mod uia;
mod visibility;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod x11;
