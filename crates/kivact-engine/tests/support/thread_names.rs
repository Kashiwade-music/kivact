//! Role: Enumerates this process's named threads for resource lifecycle tests.

#[cfg(target_os = "linux")]
pub(crate) fn names() -> Vec<String> {
    std::fs::read_dir("/proc/self/task")
        .expect("thread directory opens")
        .map(|entry| {
            std::fs::read_to_string(entry.expect("thread entry").path().join("comm"))
                .expect("thread name is readable")
                .trim_end()
                .to_owned()
        })
        .collect()
}

#[cfg(windows)]
pub(crate) use windows::names;

#[cfg(windows)]
mod windows {
    //! Role: Reads thread descriptions through owned Windows snapshot handles.
    #![allow(unsafe_code, reason = "test-only OS thread enumeration")]

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, LocalFree};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcessId, GetThreadDescription, OpenThread, THREAD_QUERY_LIMITED_INFORMATION,
    };

    struct OwnedHandle(HANDLE);

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            // SAFETY: This wrapper owns a valid snapshot or thread handle exactly once.
            unsafe { CloseHandle(self.0) };
        }
    }

    pub(crate) fn names() -> Vec<String> {
        // SAFETY: The flags request a read-only thread snapshot; no pointers are passed.
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        assert_ne!(snapshot, INVALID_HANDLE_VALUE, "thread snapshot opens");
        let snapshot = OwnedHandle(snapshot);
        // SAFETY: This call takes no arguments and only reads the process identifier.
        let process = unsafe { GetCurrentProcessId() };
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
        // SAFETY: The snapshot is live and entry has the required size and writable storage.
        let mut available = unsafe { Thread32First(snapshot.0, &mut entry) };
        while available != 0 {
            if entry.th32OwnerProcessID == process {
                // SAFETY: Query-only access to an enumerated identifier; failure is checked.
                let thread =
                    unsafe { OpenThread(THREAD_QUERY_LIMITED_INFORMATION, 0, entry.th32ThreadID) };
                if thread != 0 {
                    let thread = OwnedHandle(thread);
                    let mut description = std::ptr::null_mut();
                    // SAFETY: The handle is live and description is a writable output pointer.
                    let result = unsafe { GetThreadDescription(thread.0, &mut description) };
                    if result >= 0 && !description.is_null() {
                        let mut length = 0;
                        // SAFETY: A successful call returns a null-terminated UTF-16 allocation.
                        while unsafe { *description.wrapping_add(length) } != 0 {
                            length += 1;
                        }
                        // SAFETY: The preceding scan established this initialized UTF-16 range.
                        let units = unsafe { std::slice::from_raw_parts(description, length) };
                        names.push(String::from_utf16_lossy(units));
                        // SAFETY: GetThreadDescription transfers this LocalAlloc allocation to us.
                        unsafe { LocalFree(description.cast()) };
                    }
                }
            }
            // SAFETY: The snapshot stays live and entry remains valid writable storage.
            available = unsafe { Thread32Next(snapshot.0, &mut entry) };
        }
        names
    }
}
