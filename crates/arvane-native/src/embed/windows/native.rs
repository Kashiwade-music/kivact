//! Role: Keep Win32 identity, message ids and the host-facing Backend below dispatch.
use crate::embed::child::{Backend, Shared};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread::ThreadId;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    PostMessageW, SMTO_NORMAL, SendMessageTimeoutW, SendMessageW, WM_APP,
};
pub(super) const WAKE: u32 = WM_APP + 0x521;
pub(super) const COMMAND: u32 = WM_APP + 0x522;
pub(super) const TIMER: usize = 1;
pub(super) struct Native {
    pub(super) hwnd: AtomicIsize,
    pub(super) ui_thread: ThreadId,
    pub(super) wake_pending: AtomicBool,
}
impl Native {
    pub(super) fn hwnd(&self) -> HWND {
        self.hwnd.load(Ordering::Acquire)
    }
}
/// Largest physical dimension a Win32 child window accepts (`i32` extents).
pub(super) const MAX_DIMENSION: u32 = i32::MAX as u32;
impl Backend for Arc<Native> {
    const MAX_DIMENSION: u32 = MAX_DIMENSION;
    fn wake(&self) {
        // SAFETY: posting the opaque child HWND never accesses UI-only state.
        unsafe {
            PostMessageW(self.hwnd(), COMMAND, 0, 0);
        }
    }
    fn collect_panic(&self, _: &Shared) {
        // The App runs on the host GUI thread; its panic is kept by `wndproc`.
    }
    fn close(&self, shared: &Shared) {
        if !shared.alive() {
            return;
        }
        // `request_close` already ran; the window procedure's stop check turns
        // this notification into cleanup. A busy App callback defers the stop
        // until it returns.
        if std::thread::current().id() == self.ui_thread {
            // SAFETY: on the UI thread this is a direct call into our window procedure.
            unsafe {
                SendMessageW(self.hwnd(), COMMAND, 0, 0);
            }
        } else {
            let mut result = 0;
            // SAFETY: only an opaque HWND crosses threads; the window procedure
            // owns all UI state. The timeout bounds a UI thread that never pumps.
            let sent = unsafe {
                SendMessageTimeoutW(
                    self.hwnd(),
                    COMMAND,
                    0,
                    0,
                    SMTO_NORMAL,
                    crate::wait::SHUTDOWN_WAIT.as_millis() as u32,
                    &mut result,
                )
            };
            if sent == 0 && shared.alive() {
                crate::detached::DETACHED.store(true, Ordering::Release);
                crate::module::retain_module();
                tracing::warn!(target: "kivact", "embedded Win32 UI thread did not stop within two seconds; detaching");
            }
        }
        if shared.alive() {
            // Cleanup still runs our code after this drop returns (a busy App
            // callback defers it), so keep the module mapped even if the host
            // unloads immediately.
            crate::module::retain_module();
        }
    }
}
