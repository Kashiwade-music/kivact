//! Role: Validates the Win32 parent and creates the child window on its GUI thread.

use super::native::{COMMAND, MAX_DIMENSION, Native};
use super::state::{Phase, ScaleSource, State};
use crate::embed::child::{Handle, Shared};
use arvane_shell::{ChildOptions, MakeHandler, ShellChild, ShellError};
use std::num::NonZeroIsize;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetWindowThreadProcessId, IsWindow, PostMessageW, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SetWindowPos, ShowWindow, WS_CHILD, WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS,
};

pub(in crate::embed) fn open(
    parent: kivact_types::protocol::ParentWindow,
    options: ChildOptions,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    let kivact_types::protocol::ParentWindow::Win32(parent) = parent else {
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
    let scale = options
        .scale
        .unwrap_or_else(|| super::dpi::scale(parent.get()));
    let (width, height) = crate::embed::geometry::physical_size(
        kivact_types::Size {
            width: options.width,
            height: options.height,
        },
        scale,
        MAX_DIMENSION,
    )?;
    let guard = super::dpi::Guard::for_parent(parent.get());
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
    let Some(hwnd) = NonZeroIsize::new(hwnd) else {
        return Err(ShellError::new("CreateWindowExW failed"));
    };
    let shared = Arc::new(Shared::new(width, height, scale, options.panic));
    let native = Arc::new(Native {
        hwnd,
        ui_thread: std::thread::current().id(),
        wake_pending: AtomicBool::new(false),
    });
    let state = Rc::new(State::new(
        shared.clone(),
        native.clone(),
        instance,
        if options.scale.is_some() {
            ScaleSource::Host
        } else {
            ScaleSource::System
        },
        options.on_resize_request,
    ));
    let hwnd = hwnd.get();
    if let Err(error) = super::proc::attach(&state, hwnd) {
        state.destroy();
        return Err(error);
    }
    // From here on any early return, and a panic of the App while it starts,
    // detaches the window procedure and destroys the child.
    let opening = Opening::new(state.clone(), hwnd);
    // SAFETY: opt in only our newly created child to shell drop messages.
    unsafe { windows_sys::Win32::UI::Shell::DragAcceptFiles(hwnd, 1) };
    state.uia.replace(Some(super::bridge::adapter(
        hwnd,
        state.a11y.clone(),
        state.waker(),
    )));
    if options.scale.is_none() {
        let actual = super::dpi::scale(hwnd);
        if actual != scale {
            shared.set_scale(actual);
            let (width, height) = crate::embed::geometry::physical_size(
                kivact_types::Size {
                    width: options.width,
                    height: options.height,
                },
                actual,
                MAX_DIMENSION,
            )?;
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
    state
        .environment
        .replace(super::window::environment(&state));
    state.start(make);
    if let Some(error) = shared.failure() {
        return Err(error);
    }
    if shared.panicked() {
        return Err(ShellError::new("the embedded App panicked while opening"));
    }
    // The host may destroy the parent (and with it this child) while the App starts.
    if shared.stopped() {
        return Err(ShellError::new(
            "the parent window was destroyed while the editor opened",
        ));
    }
    // Initialize UIA before visibility: showing a window can ask for its provider.
    // SAFETY: show only this live child, retaining the host's activation.
    unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
    state.phase.set(Phase::Live);
    // Post queued start-time work: no host callback is entered during open.
    // SAFETY: the live child HWND belongs to this thread.
    unsafe {
        PostMessageW(hwnd, COMMAND, 0, 0);
    }
    opening.disarm();
    Ok(Box::new(Handle {
        shared,
        backend: native,
    }))
}

fn module() -> Result<isize, ShellError> {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleHandleExW,
    };
    let mut module = 0;
    // SAFETY: FROM_ADDRESS identifies our module without extending its lifetime.
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            super::proc::wndproc as *const () as *const u16,
            &mut module,
        )
    };
    if found == 0 {
        Err(ShellError::new("GetModuleHandleExW failed"))
    } else {
        Ok(module)
    }
}

/// Destroys a child whose opening did not finish: on an `Err` return and on
/// unwind. The window procedure is detached first, so no App code runs again.
struct Opening {
    state: Rc<State>,
    hwnd: isize,
    armed: bool,
}

impl Opening {
    fn new(state: Rc<State>, hwnd: isize) -> Self {
        Self {
            state,
            hwnd,
            armed: true,
        }
    }
    fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for Opening {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        super::a11y::disconnect(&self.state);
        super::proc::detach(&self.state, self.hwnd);
        self.state.destroy();
    }
}
