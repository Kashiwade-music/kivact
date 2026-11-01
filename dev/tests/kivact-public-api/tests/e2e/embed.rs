//! Role: Shared embedding contracts and platform-specific real-window checks.
#![cfg(any(windows, target_os = "linux"))]
mod contract;
mod support;
#[cfg(windows)]
mod uia;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod x11;
