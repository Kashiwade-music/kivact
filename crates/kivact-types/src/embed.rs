//! Role: Pure options and parent identities for embedded windows.

use crate::{GpuPreference, Size};

/// A host window's raw handle value; this type never dereferences it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParentWindow {
    /// A Win32 `HWND`.
    Win32(std::num::NonZeroIsize),
    /// An X11 window id.
    X11(std::num::NonZeroU32),
}

/// Initial embedded-window values; callbacks belong to the host's hooks.
#[derive(Clone, Debug, PartialEq)]
pub struct EmbedOptions {
    /// Initial dimensions in logical pixels.
    pub size: Size,
    /// Physical pixels per logical pixel, or the operating system's scale.
    pub scale: Option<f64>,
    /// Prefer GPU rendering when the platform supports it.
    pub prefer_gpu: bool,
    pub gpu_preference: GpuPreference,
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            size: Size {
                width: 960.0,
                height: 720.0,
            },
            scale: None,
            prefer_gpu: true,
            gpu_preference: GpuPreference::default(),
        }
    }
}
