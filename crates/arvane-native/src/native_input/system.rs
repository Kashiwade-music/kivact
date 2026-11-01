//! Role: Portable queries of system input settings and file-drag tracking.

use super::state::MouseSettings;

impl MouseSettings {
    /// The system's settings (the defaults where the shell cannot read
    /// them).
    pub(crate) fn read() -> Self {
        #[cfg(windows)]
        {
            super::windows::mouse_settings()
        }
        #[cfg(not(windows))]
        {
            MouseSettings::default()
        }
    }
}

/// Whether the system asks to reduce motion (Windows: the "animation
/// effects" setting is off); `false` where the shell cannot tell.
pub(crate) fn reduced_motion() -> bool {
    #[cfg(windows)]
    {
        super::windows::reduced_motion().unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Current file drag position; Windows queries it even during OLE capture.
pub(crate) fn file_position(window: &winit::window::Window) -> Option<kivact_types::Point> {
    #[cfg(windows)]
    {
        super::windows::file_position(window)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        None
    }
}

pub(crate) fn track_files(window: &winit::window::Window, tracking: bool) {
    #[cfg(windows)]
    {
        super::windows::track_files(window, tracking);
    }
    #[cfg(not(windows))]
    {
        let _ = (window, tracking);
    }
}
