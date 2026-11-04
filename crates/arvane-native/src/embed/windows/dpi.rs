//! Role: Thread-scoped parent DPI awareness, never process-wide settings.
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
type Proc = unsafe extern "system" fn() -> isize;
type SetContext = unsafe extern "system" fn(isize) -> isize;
type GetContext = unsafe extern "system" fn(HWND) -> isize;
type GetDpi = unsafe extern "system" fn(HWND) -> u32;
/// What undoes the thread's DPI context change.
struct Restore {
    set: SetContext,
    old: std::num::NonZeroIsize,
}
pub(super) struct Guard(Option<Restore>);
fn api(name: &[u8]) -> Option<Proc> {
    let user32: Vec<u16> = "user32.dll\0".encode_utf16().collect();
    // SAFETY: querying a loaded system module does not change its reference count.
    let module = unsafe { GetModuleHandleW(user32.as_ptr()) };
    if module == 0 {
        return None;
    }
    // SAFETY: the named export's address remains valid while user32 is loaded.
    unsafe { GetProcAddress(module, name.as_ptr()) }
}
impl Guard {
    pub(super) fn for_parent(parent: HWND) -> Self {
        let set = api(b"SetThreadDpiAwarenessContext\0").map(|f| {
            // SAFETY: this export has precisely the SetContext ABI/signature.
            unsafe { std::mem::transmute::<Proc, SetContext>(f) }
        });
        let get = api(b"GetWindowDpiAwarenessContext\0").map(|f| {
            // SAFETY: this export has precisely the GetContext ABI/signature.
            unsafe { std::mem::transmute::<Proc, GetContext>(f) }
        });
        if let (Some(set), Some(get)) = (set, get) {
            // SAFETY: parent was validated before the DPI guard was constructed.
            let context = unsafe { get(parent) };
            // SAFETY: this alters only the current UI thread and saves its old context.
            let old = unsafe { set(context) };
            Self(std::num::NonZeroIsize::new(old).map(|old| Restore { set, old }))
        } else {
            Self(None)
        }
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(restore) = &self.0 {
            // SAFETY: restore the exact previous context on this guard's creating thread.
            unsafe {
                (restore.set)(restore.old.get());
            }
        }
    }
}
pub(super) fn scale(hwnd: HWND) -> f64 {
    let dpi = api(b"GetDpiForWindow\0")
        .map(|f| {
            // SAFETY: this export has precisely the GetDpi ABI/signature.
            let get = unsafe { std::mem::transmute::<Proc, GetDpi>(f) };
            // SAFETY: GetDpiForWindow queries the opaque HWND and retains no resources.
            unsafe { get(hwnd) }
        })
        .filter(|dpi| *dpi > 0)
        .unwrap_or(96);
    dpi as f64 / 96.0
}
