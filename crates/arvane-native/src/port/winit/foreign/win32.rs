//! Role: Win32 implementation of the child window that fills a foreign window.
#![allow(
    unsafe_code,
    reason = "Win32 child-window FFI is confined to this leaf"
)]

use arvane_shell::ShellError;
use kivact_types::NativeWindow;
use std::num::NonZeroIsize;
use windows_sys::Win32::Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError, HWND};
use windows_sys::Win32::Graphics::Gdi::{BLACK_BRUSH, GetStockObject};
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleHandleExW,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GW_CHILD, GetParent, GetWindow, IsWindow,
    RegisterClassW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SetWindowPos, WNDCLASSW, WS_CHILD,
    WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_NOPARENTNOTIFY, WS_VISIBLE,
};

const CLASS_NAME: [u16; 20] = [
    b'K' as u16,
    b'i' as u16,
    b'v' as u16,
    b'a' as u16,
    b'c' as u16,
    b't' as u16,
    b'F' as u16,
    b'o' as u16,
    b'r' as u16,
    b'e' as u16,
    b'i' as u16,
    b'g' as u16,
    b'n' as u16,
    b'W' as u16,
    b'i' as u16,
    b'n' as u16,
    b'd' as u16,
    b'o' as u16,
    b'w' as u16,
    0,
];

#[expect(
    clippy::disallowed_types,
    reason = "process-wide foreign child window class registered once"
)]
static REGISTERED: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();

/// The loop's shared state for foreign children (nothing on Win32).
#[derive(Default)]
pub(crate) struct ForeignConnections;

impl ForeignConnections {
    /// No connection state on Win32.
    pub(crate) fn new() -> Self {
        Self
    }
}

/// The Win32 child window of one foreign window.
pub(crate) struct ForeignOs {
    child: HWND,
    /// The winit window that the child was created under.
    parent: HWND,
}

impl ForeignOs {
    /// Create the child of `window`, `width` x `height` physical pixels.
    pub(crate) fn create(
        window: &winit::window::Window,
        width: u32,
        height: u32,
        _connections: &mut ForeignConnections,
    ) -> Result<Self, ShellError> {
        register_class()?;
        let parent = parent_hwnd(window)?;
        let instance = module_instance()?;
        // SAFETY: the class is registered, the parent is the live winit window
        // and the size is a positive physical extent.
        let child = unsafe {
            CreateWindowExW(
                WS_EX_NOPARENTNOTIFY,
                CLASS_NAME.as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0,
                0,
                width.max(1) as i32,
                height.max(1) as i32,
                parent,
                0,
                instance,
                std::ptr::null(),
            )
        };
        if child == 0 {
            return Err(ShellError::new(format!(
                "the foreign child window could not be created: {}",
                last_error()
            )));
        }
        Ok(Self { child, parent })
    }

    /// Resize the child to `width` x `height` physical pixels.
    pub(crate) fn fit(&mut self, width: u32, height: u32) -> Result<(), ShellError> {
        // SAFETY: the child is ours and the size is a positive physical extent.
        let ok = unsafe {
            SetWindowPos(
                self.child,
                0,
                0,
                0,
                width.max(1) as i32,
                height.max(1) as i32,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOMOVE,
            )
        };
        if ok == 0 {
            return Err(ShellError::new(format!(
                "the foreign child window could not be resized: {}",
                last_error()
            )));
        }
        Ok(())
    }

    /// Give the keyboard focus to the foreign UI's own child, if it has one.
    pub(crate) fn focus_into(&self) {
        // SAFETY: the handle is opaque and stays live while the window is open.
        let focus = unsafe { GetWindow(self.child, GW_CHILD) };
        if focus != 0 {
            // SAFETY: the focused handle is the foreign UI's own child window.
            unsafe {
                SetFocus(focus);
            }
        }
    }

    /// The raw child handle given to the foreign UI as its parent.
    pub(crate) fn native(&self) -> NativeWindow {
        let child = NonZeroIsize::new(self.child).expect("invariant: a created child is nonzero");
        NativeWindow::Win32(child)
    }
}

impl Drop for ForeignOs {
    fn drop(&mut self) {
        // SAFETY: IsWindow validates the opaque handle without dereferencing it.
        if unsafe { IsWindow(self.child) } == 0 {
            return;
        }
        // SAFETY: GetParent reads the parent link of the handle validated above.
        // A destroyed child's handle may now name another window; only destroy
        // the child that is still ours.
        if unsafe { GetParent(self.child) } != self.parent {
            return;
        }
        // SAFETY: the child is still our window under our parent, so destroying it is
        // its only cleanup.
        unsafe {
            DestroyWindow(self.child);
        }
    }
}

/// Register the child class once per process.
fn register_class() -> Result<(), ShellError> {
    match REGISTERED.get_or_init(register) {
        Ok(()) => Ok(()),
        Err(reason) => Err(ShellError::new(reason.clone())),
    }
}

fn register() -> Result<(), String> {
    let instance = module_instance().map_err(|error| error.to_string())?;
    let class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(DefWindowProcW),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: 0,
        hCursor: 0,
        // SAFETY: GetStockObject returns a process-wide stock brush.
        hbrBackground: unsafe { GetStockObject(BLACK_BRUSH) } as isize,
        lpszMenuName: std::ptr::null(),
        lpszClassName: CLASS_NAME.as_ptr(),
    };
    // SAFETY: the class name outlives every window of the class and the
    // procedure lives in user32.
    let atom = unsafe { RegisterClassW(&class) };
    if atom == 0 {
        // A reload of this module in the same process finds the class its earlier
        // load registered. Windows looks classes up by module handle, so creating
        // the child still fails if the reload maps at a different handle; that
        // failure is reported where the child is created.
        if last_error_code() == ERROR_CLASS_ALREADY_EXISTS {
            return Ok(());
        }
        return Err(format!(
            "the foreign child window class could not be registered: {}",
            last_error()
        ));
    }
    Ok(())
}

fn parent_hwnd(window: &winit::window::Window) -> Result<HWND, ShellError> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window
        .window_handle()
        .map_err(|error| ShellError::new(error.to_string()))?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Err(ShellError::unsupported(
            "the window has no Win32 handle for a foreign child",
        ));
    };
    Ok(handle.hwnd.get())
}

fn module_instance() -> Result<windows_sys::Win32::Foundation::HINSTANCE, ShellError> {
    static ADDRESS: u8 = 0;
    let mut instance = 0;
    // SAFETY: the address names this module; the unchanged reference count
    // keeps the module mapped for the life of the class.
    let ok = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            (&ADDRESS as *const u8).cast(),
            &mut instance,
        )
    };
    if ok == 0 {
        return Err(ShellError::new(format!(
            "the module handle for the foreign child class is unavailable: {}",
            last_error()
        )));
    }
    Ok(instance)
}

fn last_error_code() -> u32 {
    // SAFETY: GetLastError reads this thread's error code.
    unsafe { GetLastError() }
}

fn last_error() -> String {
    format!("Win32 error {}", last_error_code())
}
