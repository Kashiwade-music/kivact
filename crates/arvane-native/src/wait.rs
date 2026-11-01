//! Role: Bounded shutdown waits on a native window's owning thread.
//!
//! Windows dispatches only synchronous sent messages, but PeekMessage delivers
//! every sent message queued for the thread, including those for host windows
//! (cross-thread sends from the host, UIA requests), not only a swapchain
//! worker's shutdown send. Posted messages and input remain with the host
//! loop, preventing application callback reentry during shutdown.
#![allow(
    unsafe_code,
    reason = "Win32 dispatches sent messages during GPU shutdown"
)]

use std::time::{Duration, Instant};

#[cfg(feature = "embed")]
pub(crate) const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);

pub(crate) fn until(done: &mut dyn FnMut() -> bool, limit: Duration) -> bool {
    let started = Instant::now();
    loop {
        if done() {
            return true;
        }
        if started.elapsed() >= limit {
            return false;
        }
        tick();
    }
}

#[cfg(not(windows))]
fn tick() {
    std::thread::sleep(Duration::from_millis(1));
}

#[cfg(windows)]
fn tick() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MSG, MsgWaitForMultipleObjectsEx, PM_NOREMOVE, PM_QS_SENDMESSAGE, PeekMessageW,
        QS_SENDMESSAGE,
    };
    // SAFETY: No handles are passed; the call waits at most one millisecond.
    unsafe { MsgWaitForMultipleObjectsEx(0, std::ptr::null(), 1, QS_SENDMESSAGE, 0) };
    let mut message: MSG = MSG {
        hwnd: 0,
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: windows_sys::Win32::Foundation::POINT { x: 0, y: 0 },
    };
    // SAFETY: The message buffer is valid. This dispatches sent messages
    // without removing input or posted messages from the owning thread.
    unsafe { PeekMessageW(&mut message, 0, 0, 0, PM_NOREMOVE | PM_QS_SENDMESSAGE) };
}
