//! Role: Thread-scoped parent DPI awareness, never process-wide settings.
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT, GetDpiForWindow, GetWindowDpiAwarenessContext,
    SetThreadDpiAwarenessContext,
};
/// What undoes the thread's DPI context change.
struct Restore {
    old: DPI_AWARENESS_CONTEXT,
}
pub(super) struct Guard(Option<Restore>);
impl Guard {
    pub(super) fn for_parent(parent: HWND) -> Self {
        // SAFETY: parent was validated before the DPI guard was constructed.
        let context = unsafe { GetWindowDpiAwarenessContext(parent) };
        // SAFETY: this alters only the current UI thread and saves its old context.
        let old = unsafe { SetThreadDpiAwarenessContext(context) };
        if old == 0 {
            Self(None)
        } else {
            Self(Some(Restore { old }))
        }
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(restore) = &self.0 {
            // SAFETY: restore the exact previous context on this guard's creating thread.
            unsafe { SetThreadDpiAwarenessContext(restore.old) };
        }
    }
}
pub(super) fn scale(hwnd: HWND) -> f64 {
    // SAFETY: GetDpiForWindow queries the opaque HWND and retains no resources.
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let dpi = if dpi > 0 { dpi } else { 96 };
    dpi as f64 / 96.0
}
