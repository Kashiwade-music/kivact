//! Role: Query the X11 pointer over a window for file drags.

use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use x11rb::protocol::xproto::ConnectionExt;

/// The pointer's logical position in the window, which winit does not report during an XDND
/// drag; `None` on another window system or when the pointer is on another screen.
pub(super) fn file_position(window: &winit::window::Window) -> Option<kivact_types::Point> {
    let xid = match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Xlib(handle) => handle.window as u32,
        RawWindowHandle::Xcb(handle) => handle.window.get(),
        _ => return None,
    };
    let (connection, _) = x11rb::connect(None).ok()?;
    let reply = connection.query_pointer(xid).ok()?.reply().ok()?;
    if !reply.same_screen {
        return None;
    }
    let scale = window.scale_factor();
    Some(kivact_types::Point {
        x: (f64::from(reply.win_x) / scale) as f64,
        y: (f64::from(reply.win_y) / scale) as f64,
    })
}
