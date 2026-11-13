//! Role: Validates the Win32 parent and creates the child window on its GUI thread.

use super::native::{COMMAND, MAX_DIMENSION, Native};
use super::state::{ScaleSource, State};
use crate::port::embed::child::{Handle, Shared};
use arvane_shell::{
    ChildHooks, ChildStop, EmbedOptions, MakeHandler, NoticeSink, ShellChild, ShellError,
};
use std::num::NonZeroIsize;
use std::rc::Rc;
use std::sync::Arc;
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, GetWindowThreadProcessId, IsWindow, PostMessageW, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SendMessageW, SetWindowPos, ShowWindow, WS_CHILD,
    WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
};

pub(in crate::port) fn open(
    parent: kivact_types::NativeWindow,
    options: EmbedOptions,
    hooks: ChildHooks,
    make: MakeHandler,
) -> Result<Box<dyn ShellChild>, ShellError> {
    let kivact_types::NativeWindow::Win32(parent) = parent else {
        return Err(ShellError::unsupported(
            "Windows embedding needs a Win32 parent",
        ));
    };
    validate_parent(parent)?;
    validate_options(&options)?;
    let instance = module()?;
    let scale = initial_scale(&options, parent);
    let (width, height) = crate::port::embed::geometry::physical_size(
        kivact_types::Size {
            width: options.size.width,
            height: options.size.height,
        },
        scale,
        MAX_DIMENSION,
    )?;
    let hwnd = create_child(parent, instance, width, height)?;
    let shared = Arc::new(Shared::new(
        width,
        height,
        scale,
        options.visibility,
        hooks.panic,
        NoticeSink::new(hooks.on_notice),
    ));
    let native = Arc::new(Native {
        hwnd,
        ui_thread: std::thread::current().id(),
        wake_pending: super::native::WakeCoalescer::default(),
    });
    let hwnd = hwnd.get();
    let state = Rc::new(State::new(
        shared.clone(),
        native.clone(),
        instance,
        if options.scale.is_some() {
            ScaleSource::Host
        } else {
            ScaleSource::System
        },
        options.visibility,
        options.unhandled_keys,
        hooks.on_resize_request,
        make,
    ));
    if let Err(error) = super::proc::attach(&state, hwnd) {
        state.destroy();
        return Err(error);
    }
    // Any early return or App panic from here on detaches and destroys the child.
    let opening = Opening::new(state.clone(), hwnd);
    // SAFETY: opt in only our newly created child to shell drop messages.
    unsafe { windows_sys::Win32::UI::Shell::DragAcceptFiles(hwnd, 1) };
    state
        .uia
        .replace(super::state::Uia::Active(super::bridge::adapter(
            hwnd,
            state.a11y.clone(),
            state.waker(),
        )));
    apply_actual_scale(&state, &shared, hwnd, &options, scale)?;
    state
        .environment
        .replace(super::window::environment(&state));
    // Run the root inside the window procedure's panic boundary (a same-thread send).
    // SAFETY: the live child HWND belongs to this thread; the message only
    // starts the App through our own window procedure.
    unsafe {
        SendMessageW(hwnd, super::native::START, 0, 0);
    }
    if let Some(error) = opening_stop(&shared) {
        return Err(error);
    }
    // Initialize UIA before visibility: showing a window can ask for its provider.
    show_if_visible(hwnd, options.visibility);
    state.session.opened();
    // Post start-time work without entering a host callback; SAFETY: live child HWND on this thread.
    unsafe {
        PostMessageW(hwnd, COMMAND, 0, 0);
    }
    opening.disarm();
    Ok(Box::new(Handle {
        shared,
        backend: native,
    }))
}

/// The host scale when supplied, otherwise the OS scale for `parent`.
fn initial_scale(options: &EmbedOptions, parent: NonZeroIsize) -> f64 {
    options
        .scale
        .unwrap_or_else(|| super::dpi::scale(parent.get()))
}

/// Show the child only when the embed options ask for it.
fn show_if_visible(hwnd: isize, visibility: kivact_types::embed::Visibility) {
    if visibility == kivact_types::embed::Visibility::Shown {
        // SAFETY: show only this live child, retaining the host's activation.
        unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
    }
}

/// Why an App that did not survive `START` cannot be returned, if it stopped.
fn opening_stop(shared: &Shared) -> Option<ShellError> {
    match shared.stop() {
        Some(ChildStop::Failed(error)) => Some(error),
        Some(ChildStop::Panicked) => {
            Some(ShellError::new("the embedded App panicked while opening"))
        }
        Some(ChildStop::Closed) => Some(ShellError::new(
            "the parent window was destroyed while the editor opened",
        )),
        None => None,
    }
}

/// Require the parent to belong to the calling GUI thread.
fn validate_parent(parent: NonZeroIsize) -> Result<(), ShellError> {
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
    Ok(())
}

/// Create the system child under the parent's DPI context.
fn create_child(
    parent: NonZeroIsize,
    instance: isize,
    width: u32,
    height: u32,
) -> Result<NonZeroIsize, ShellError> {
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
    Ok(hwnd)
}

/// Match the physical child extent to the DPI actually assigned by Windows.
fn apply_actual_scale(
    state: &State,
    shared: &Shared,
    hwnd: isize,
    options: &EmbedOptions,
    scale: f64,
) -> Result<(), ShellError> {
    if options.scale.is_none() {
        let actual = super::dpi::scale(hwnd);
        if actual != scale {
            shared.set_scale(actual);
            let (width, height) = crate::port::embed::geometry::physical_size(
                kivact_types::Size {
                    width: options.size.width,
                    height: options.size.height,
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
    Ok(())
}

/// Validate logical dimensions before converting to native coordinates.
fn validate_options(options: &EmbedOptions) -> Result<(), ShellError> {
    if !options.size.width.is_finite()
        || !options.size.height.is_finite()
        || options.size.width <= 0.0
        || options.size.height <= 0.0
        || options.scale.is_some_and(|s| !s.is_finite() || s <= 0.0)
    {
        return Err(ShellError::new(
            "child size and scale must be positive and finite",
        ));
    }
    Ok(())
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

enum OpeningGuard {
    Armed,
    Disarmed,
}

/// Destroys a child whose opening did not finish: on an `Err` return and on
/// unwind. The window procedure is detached first, so no App code runs again.
struct Opening {
    state: Rc<State>,
    hwnd: isize,
    guard: OpeningGuard,
}

impl Opening {
    fn new(state: Rc<State>, hwnd: isize) -> Self {
        Self {
            state,
            hwnd,
            guard: OpeningGuard::Armed,
        }
    }
    fn disarm(mut self) {
        self.guard = OpeningGuard::Disarmed;
    }
}

impl Drop for Opening {
    fn drop(&mut self) {
        if matches!(self.guard, OpeningGuard::Disarmed) {
            return;
        }
        super::a11y::disconnect(&self.state);
        // A child already destroyed (its parent went away) detached itself in
        // WM_NCDESTROY, and its HWND value may now name another window.
        if self.state.shared.alive() {
            super::proc::detach(&self.state, self.hwnd);
            self.state.destroy();
        }
    }
}
