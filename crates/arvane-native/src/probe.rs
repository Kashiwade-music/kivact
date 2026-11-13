//! Role: Answers whether a native window handle is still alive (test feature).
#![allow(unsafe_code, reason = "the Win32 probe is one opaque handle query")]

use kivact_types::NativeWindow;

/// Whether the OS still has `window` open.
#[cfg(windows)]
pub fn native_window_alive(window: NativeWindow) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindow;

    let NativeWindow::Win32(hwnd) = window else {
        return false;
    };
    // SAFETY: IsWindow reads the opaque handle value only.
    unsafe { IsWindow(hwnd.get()) != 0 }
}

/// Whether the X server still has `window` open.
#[cfg(target_os = "linux")]
pub fn native_window_alive(window: NativeWindow) -> bool {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::ConnectionExt;
    use x11rb::xcb_ffi::XCBConnection;

    let NativeWindow::X11(id) = window else {
        return false;
    };
    let Ok((connection, _)) = XCBConnection::connect(None) else {
        return false;
    };
    let alive = connection
        .get_window_attributes(id.get())
        .is_ok_and(|cookie| cookie.reply().is_ok());
    let _ = connection.flush();
    alive
}

/// Platforms without native child windows have none to report.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn native_window_alive(window: NativeWindow) -> bool {
    let _ = window;
    false
}
