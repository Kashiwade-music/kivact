//! Role: Keep library code mapped while its threads or providers may still run.
//!
//! Retention is process-lifetime and attempted once. The caller identifies the
//! detached thread or library that needs it. An executable already has this
//! lifetime; its diagnostic distinguishes that fact from successful DLL pinning.
//! Concurrent callers wait for the loader result. Reporting happens afterward,
//! outside initialization, so a subscriber may safely reenter retain_module.
#![allow(
    unsafe_code,
    reason = "retain the native code module through the operating system loader"
)]

#[cfg(any(windows, target_os = "linux"))]
#[cfg(any(windows, target_os = "linux"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(any(windows, target_os = "linux"))]
#[expect(
    clippy::disallowed_types,
    reason = "process-wide module mapping initialized once"
)]
static MAPPING: std::sync::OnceLock<Option<&'static str>> = std::sync::OnceLock::new();
#[cfg(any(windows, target_os = "linux"))]
static REPORTED: AtomicBool = AtomicBool::new(false);

/// Keep this code module mapped until process exit for detached threads or
/// native providers. Repeated calls do nothing. Other platforms do nothing.
#[track_caller]
pub fn retain_module() {
    #[cfg(any(windows, target_os = "linux"))]
    {
        let mapping = MAPPING.get_or_init(retain);
        // Publish the report flag before entering subscriber code: a reentrant
        // call observes the completed mapping and must not log recursively.
        if REPORTED.swap(true, Ordering::AcqRel) {
            return;
        }
        let caller = std::panic::Location::caller();
        if let Some(mapping) = mapping {
            tracing::warn!(target: "kivact", %caller, mapping,
                "module code kept mapped until process exit for a detached thread or native library");
        } else {
            tracing::error!(target: "kivact", %caller,
                "could not keep the module containing retained native code mapped");
        }
    }
}

#[cfg(windows)]
fn retain() -> Option<&'static str> {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_PIN, GetModuleHandleExW,
    };
    let mut module = 0;
    // SAFETY: FROM_ADDRESS interprets the pointer as this function's code
    // address, never as UTF-16; module points to initialized writable storage.
    let pinned = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            retain_module as *const () as *const u16,
            &mut module,
        )
    };
    if pinned != 0 {
        Some("loader pin")
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
fn retain() -> Option<&'static str> {
    let mut info = std::mem::MaybeUninit::<libc::Dl_info>::uninit();
    // SAFETY: the address belongs to this live function; dladdr initializes
    // the output structure when it reports success.
    let found = unsafe { libc::dladdr(retain_module as *const () as *const _, info.as_mut_ptr()) };
    if found == 0 {
        return None;
    }
    // SAFETY: successful dladdr initialized every field of Dl_info.
    let info = unsafe { info.assume_init() };
    if info.dli_fname.is_null() {
        return None;
    }
    // SAFETY: dladdr supplied a NUL-terminated path for this live module.
    let path = unsafe { std::ffi::CStr::from_ptr(info.dli_fname) };
    // SAFETY: the path remains valid while this function's module is loaded;
    // NOLOAD only retains that loaded module and NODELETE prevents unloading.
    let module = unsafe {
        libc::dlopen(
            info.dli_fname,
            libc::RTLD_NOW | libc::RTLD_NOLOAD | libc::RTLD_NODELETE,
        )
    };
    if !module.is_null() {
        // Intentionally never dlclose: this reference is process-lifetime.
        return Some("loader NODELETE");
    }
    // A statically linked executable cannot be dlopened, and is already
    // mapped for the process lifetime. Do not claim dlopen succeeded.
    use std::os::unix::ffi::OsStrExt;
    let module_path = std::path::Path::new(std::ffi::OsStr::from_bytes(path.to_bytes()));
    let executable = std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok());
    if executable.is_some() && module_path.canonicalize().ok() == executable {
        Some("executable already mapped for process lifetime")
    } else {
        None
    }
}
