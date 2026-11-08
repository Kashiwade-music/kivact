//! Role: Portable queries of system input settings and file-drag tracking.

/// The system's pointer settings, in logical pixels and nanoseconds.
pub(crate) fn pointer_settings(scale: f64) -> kivact_types::PointerSettings {
    #[cfg(windows)]
    {
        super::windows::pointer_settings(scale)
    }
    #[cfg(not(windows))]
    {
        let _ = scale;
        kivact_types::PointerSettings::default()
    }
}

/// Whether the system asks to reduce motion (Windows: the "animation
/// effects" setting is off; Linux: the settings portal); `false` where the
/// shell cannot tell.
pub(crate) fn reduced_motion() -> bool {
    #[cfg(windows)]
    {
        super::windows::reduced_motion().unwrap_or(false)
    }
    #[cfg(target_os = "linux")]
    {
        crate::port::linux_settings::reduced_motion()
    }
    #[cfg(not(any(windows, target_os = "linux")))]
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
