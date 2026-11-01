//! Role: Enumerate this process's thread descriptions through Windows APIs.
#![allow(
    unsafe_code,
    reason = "Toolhelp and thread-description FFI is confined to test support"
)]

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, LocalFree};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
};
use windows_sys::Win32::System::Threading::{
    GetThreadDescription, OpenThread, THREAD_QUERY_LIMITED_INFORMATION,
};

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: This wrapper owns a valid snapshot or thread handle exactly once.
        unsafe { CloseHandle(self.0) };
    }
}

pub(super) fn thread_names() -> Vec<String> {
    // SAFETY: Snapshot flags require no pointers; process zero selects the current system.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    assert_ne!(snapshot, INVALID_HANDLE_VALUE);
    let snapshot = Handle(snapshot);
    let mut entry = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
        cntUsage: 0,
        th32ThreadID: 0,
        th32OwnerProcessID: 0,
        tpBasePri: 0,
        tpDeltaPri: 0,
        dwFlags: 0,
    };
    let mut names = Vec::new();
    // SAFETY: entry has the required size and remains writable for this call.
    let mut available = unsafe { Thread32First(snapshot.0, &mut entry) };
    while available != 0 {
        if entry.th32OwnerProcessID == std::process::id() {
            // SAFETY: Opening an enumerated ID does not borrow memory; failure is checked.
            let thread =
                unsafe { OpenThread(THREAD_QUERY_LIMITED_INFORMATION, 0, entry.th32ThreadID) };
            if thread != 0 {
                let thread = Handle(thread);
                let mut description = std::ptr::null_mut();
                // SAFETY: thread is valid and description points to writable pointer storage.
                let result = unsafe { GetThreadDescription(thread.0, &mut description) };
                if result >= 0 && !description.is_null() {
                    let mut length = 0;
                    loop {
                        // SAFETY: The successful call returns a null-terminated UTF-16 allocation.
                        let unit = unsafe { description.add(length) };
                        // SAFETY: unit is inside that allocation, including its terminating null.
                        if unsafe { *unit } == 0 {
                            break;
                        }
                        length += 1;
                    }
                    // SAFETY: length was measured within the returned allocation.
                    let units = unsafe { std::slice::from_raw_parts(description, length) };
                    names.push(String::from_utf16_lossy(units));
                    // SAFETY: GetThreadDescription allocates with LocalAlloc and transfers ownership.
                    unsafe { LocalFree(description as _) };
                }
            }
        }
        entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
        // SAFETY: snapshot remains valid; entry has the required size and writable storage.
        available = unsafe { Thread32Next(snapshot.0, &mut entry) };
    }
    names
}

#[cfg(feature = "embed")]
pub(crate) fn gui_resources() -> (u32, u32) {
    use windows_sys::Win32::System::Threading::{
        GR_GDIOBJECTS, GR_USEROBJECTS, GetCurrentProcess, GetGuiResources,
    };
    // SAFETY: GetCurrentProcess returns this process's valid pseudo-handle.
    let process = unsafe { GetCurrentProcess() };
    // SAFETY: process is live and GR_GDIOBJECTS requests a process-local counter.
    let gdi = unsafe { GetGuiResources(process, GR_GDIOBJECTS) };
    // SAFETY: process is live and GR_USEROBJECTS requests a process-local counter.
    let user = unsafe { GetGuiResources(process, GR_USEROBJECTS) };
    (gdi, user)
}
