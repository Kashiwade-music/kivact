//! Role: Win32 embedded children, owned by the host GUI thread.
//!
//! The Send handle owns only atomic/locked command state. Rc state never
//! crosses threads. Every wndproc entry retains it independently; destruction
//! restores the system window procedure and detaches USERDATA before returning.
//! A system-owned STATIC class avoids DLL class registration surviving a
//! parent-driven destruction that drops the last handle from its own cleanup.
#![allow(
    unsafe_code,
    reason = "Win32 child-window FFI is confined to the native backend"
)]

mod a11y;
mod bridge;
mod cpu;
mod dpi;
mod ime;
mod input;
mod native;
mod proc;
mod services;
mod state;
mod target;
#[cfg(feature = "testing")]
mod test_parent;
mod window;
#[cfg(feature = "testing")]
pub use test_parent::TestParent;

use super::child::{Handle, Shared};
use arvane_shell::{ChildOptions, MakeHandler, ShellChild, ShellError};
use native::{COMMAND, MAX_DIMENSION, Native};
use state::State;
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub(super) fn open(
    options: ChildOptions,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    let ui_types::ParentWindow::Win32(parent) = options.parent else {
        return Err(ShellError::unsupported(
            "Windows embedding needs a Win32 parent",
        ));
    };
    // SAFETY: IsWindow validates the opaque host value without dereferencing it.
    if unsafe { IsWindow(parent.get()) } == 0 {
        return Err(ShellError::new("the parent window is not a window"));
    }
    let mut parent_process = 0;
    // SAFETY: the parent was validated above; the call writes only the local pid.
    let parent_thread = unsafe { GetWindowThreadProcessId(parent.get(), &mut parent_process) };
    // SAFETY: the getter reads the calling thread identity only.
    let thread = unsafe { GetCurrentThreadId() };
    // SAFETY: the getter reads the calling process identity only.
    let process = unsafe { GetCurrentProcessId() };
    if parent_thread != thread || parent_process != process {
        return Err(ShellError::new(
            "the parent window must belong to the calling GUI thread",
        ));
    }
    if !options.width.is_finite()
        || !options.height.is_finite()
        || options.width <= 0.0
        || options.height <= 0.0
        || options.scale.is_some_and(|s| !s.is_finite() || s <= 0.0)
    {
        return Err(ShellError::new(
            "child size and scale must be positive and finite",
        ));
    }
    let instance = module()?;
    let scale = options.scale.unwrap_or_else(|| dpi::scale(parent.get()));
    let (width, height) = super::geometry::physical_size(
        ui_types::Size {
            width: options.width,
            height: options.height,
        },
        scale,
        MAX_DIMENSION,
    )?;
    let shared = Arc::new(Shared::new(width, height, scale));
    let native = Arc::new(Native {
        hwnd: 0.into(),
        ui_thread: std::thread::current().id(),
        wake_pending: AtomicBool::new(false),
    });
    let state = Rc::new(State::new(
        shared.clone(),
        native.clone(),
        instance,
        options.scale.is_some(),
        options.on_resize_request,
    ));
    let guard = dpi::Guard::for_parent(parent.get());
    // SAFETY: STATIC is a system-owned class; creation receives no Rust state.
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            [83u16, 84, 65, 84, 73, 67, 0].as_ptr(),
            [0u16].as_ptr(),
            WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            0,
            0,
            width as i32,
            height as i32,
            parent.get(),
            0,
            instance,
            std::ptr::null(),
        )
    };
    drop(guard);
    if hwnd == 0 {
        shared.alive.store(false, Ordering::Release);
        return Err(ShellError::new("CreateWindowExW failed"));
    }
    native.hwnd.store(hwnd, Ordering::Release);
    if let Err(error) = proc::attach(&state, hwnd) {
        state.destroy();
        return Err(error);
    }
    // SAFETY: opt in only our newly created child to shell drop messages.
    unsafe { windows_sys::Win32::UI::Shell::DragAcceptFiles(hwnd, 1) };
    state.a11y.replace(Some(bridge::Bridge::new(
        hwnd,
        state.waker(),
        f64::from(scale),
    )));
    if options.scale.is_none() {
        let actual = dpi::scale(hwnd);
        if actual != scale {
            shared.set_scale_value(actual);
            let (width, height) = match super::geometry::physical_size(
                ui_types::Size {
                    width: options.width,
                    height: options.height,
                },
                actual,
                MAX_DIMENSION,
            ) {
                Ok(size) => size,
                Err(error) => {
                    state.destroy();
                    return Err(error);
                }
            };
            // SAFETY: this child belongs to the opening GUI thread; only its
            // physical extent changes to match the actual child DPI.
            unsafe {
                SetWindowPos(
                    hwnd,
                    0,
                    0,
                    0,
                    width as i32,
                    height as i32,
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            state.geometry();
        }
    }
    state.environment.replace(window::environment(&state));
    state.start(make);
    let failure = shared.failure();
    if let Some(error) = failure {
        state.destroy();
        return Err(error);
    }
    // The host may destroy the parent (and with it this child) while the App starts.
    if !shared.alive.load(Ordering::Acquire) || state.closing.get() {
        return Err(ShellError::new(
            "the parent window was destroyed while the editor opened",
        ));
    }
    // Initialize UIA before visibility: showing a window can ask for its provider.
    // SAFETY: show only this live child, retaining the host's activation.
    unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
    state.opening.set(false);
    // Post queued start-time work: no host callback is entered during open.
    // SAFETY: the live child HWND belongs to this thread.
    unsafe {
        PostMessageW(hwnd, COMMAND, 0, 0);
    }
    Ok(Box::new(Handle {
        shared,
        backend: native,
    }))
}

fn module() -> Result<isize, ShellError> {
    use windows_sys::Win32::System::LibraryLoader::*;
    let mut module = 0;
    // SAFETY: FROM_ADDRESS identifies our module without extending its lifetime.
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            proc::wndproc as *const () as *const u16,
            &mut module,
        )
    };
    if found == 0 {
        Err(ShellError::new("GetModuleHandleExW failed"))
    } else {
        Ok(module)
    }
}
