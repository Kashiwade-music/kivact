//! Role: Read the Windows monitor work area for initial window placement.

#![allow(
    unsafe_code,
    reason = "Win32 monitor work areas require GetMonitorInfoW FFI"
)]

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO};
use winit::monitor::MonitorHandle;
use winit::platform::windows::MonitorHandleExtWindows;

pub(super) fn work_area(monitor: &MonitorHandle) -> Option<super::placement::WorkArea> {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        rcWork: RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        dwFlags: 0,
    };
    // SAFETY: winit supplies a live monitor handle; info is writable and declares its size.
    if unsafe { GetMonitorInfoW(monitor.hmonitor() as _, &mut info) } == 0 {
        return None;
    }
    let r = info.rcWork;
    let width = u32::try_from(i64::from(r.right) - i64::from(r.left)).ok()?;
    let height = u32::try_from(i64::from(r.bottom) - i64::from(r.top)).ok()?;
    (width > 0 && height > 0).then_some((r.left, r.top, width, height))
}
