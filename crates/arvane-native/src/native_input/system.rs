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

/// Current file drag position from the OS (Windows even during OLE capture, X11 by a pointer
/// query); `None` where the platform cannot tell.
pub(crate) fn file_position(window: &winit::window::Window) -> Option<kivact_types::Point> {
    #[cfg(windows)]
    {
        super::windows::file_position(window)
    }
    #[cfg(target_os = "linux")]
    {
        super::x11::file_position(window)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
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
