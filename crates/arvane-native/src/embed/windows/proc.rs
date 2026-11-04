//! Role: Win32 dispatch with an independently retained Rc on every entry.
use super::state::{Phase, ScaleSource, State};
use arvane_shell::{ShellError, ShellEvent};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, SetLastError, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{BeginPaint, EndPaint};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DLGC_WANTALLKEYS, DLGC_WANTCHARS, DefWindowProcW, GWLP_USERDATA, GWLP_WNDPROC,
    GetWindowLongPtrW, HTCLIENT, KillTimer, SetWindowLongPtrW, WM_DESTROY,
    WM_DPICHANGED_AFTERPARENT, WM_DROPFILES, WM_ERASEBKGND, WM_GETDLGCODE, WM_GETOBJECT,
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_REQUEST, WM_IME_SETCONTEXT,
    WM_IME_STARTCOMPOSITION, WM_KILLFOCUS, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_MBUTTONDBLCLK,
    WM_MBUTTONDOWN, WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN,
    WM_SETCURSOR, WM_SETFOCUS, WM_SIZE, WM_TIMER, WM_XBUTTONDBLCLK, WM_XBUTTONDOWN, WNDPROC,
};

/// The window procedure of the embedded child window.
///
/// # Safety
///
/// Only Windows may call this, as the window procedure that `attach`
/// installed on a child window, on the thread that created `hwnd`, with the
/// arguments of the message it is delivering. The window's `GWLP_USERDATA`
/// slot must be zero or hold the `State` pointer that `attach` stored there.
pub(super) unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    // SAFETY: only this proc writes the child HWND's USERDATA slot.
    let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const State;
    if raw.is_null() {
        // SAFETY: messages without attached state use standard Win32 processing.
        return unsafe { DefWindowProcW(hwnd, message, w, l) };
    }
    // SAFETY: USERDATA owns a live Rc on this UI thread. Retain independently of
    // a nested destruction detaching that reference during the callback.
    unsafe {
        Rc::increment_strong_count(raw);
    }
    // SAFETY: this consumes precisely the increment performed for this entry.
    let state = unsafe { Rc::from_raw(raw) };
    let result = match catch_unwind(AssertUnwindSafe(|| dispatch(&state, hwnd, message, w, l))) {
        Ok(result) => result,
        Err(payload) => {
            // A panic must not unwind into the host's message loop. Keep it:
            // the child stops below without running more App code, and the
            // host's next call on `Embedded` resumes it.
            state.shared.keep_panic(payload);
            0
        }
    };
    if state.shared.stopped()
        && !state.in_callback.get()
        && matches!(state.phase.get(), Phase::Opening | Phase::Live)
    {
        state.stop();
    }
    if message == WM_NCDESTROY {
        state.shared.window_gone();
        detach(&state, hwnd);
        return original(&state, hwnd, message, w, l);
    }
    result
}
fn dispatch(state: &Rc<State>, hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match message {
        WM_GETOBJECT => super::a11y::get_object(state, w, l)
            .unwrap_or_else(|| original(state, hwnd, message, w, l)),
        WM_DROPFILES => {
            super::services::dropped(state, w as isize);
            0
        }
        WM_IME_SETCONTEXT => {
            // SAFETY: let Windows manage the IME context, with composition drawn by the App.
            unsafe {
                CallWindowProcW(
                    state.original_proc.get(),
                    hwnd,
                    message,
                    w,
                    l & !(windows_sys::Win32::UI::Input::Ime::ISC_SHOWUICOMPOSITIONWINDOW as isize),
                )
            }
        }
        WM_IME_REQUEST => {
            state.input.scale.set(state.shared.scale());
            let result = crate::native_input::ime_request(hwnd, &state.input, w, l);
            if let Some((start, end)) = state.input.reconvert.take() {
                state.event(ShellEvent::Ime(kivact_types::ImeInput::Selection(
                    start..end,
                )));
            }
            result.unwrap_or_else(|| original_if_alive(state, hwnd, message, w, l))
        }
        WM_IME_STARTCOMPOSITION | WM_IME_COMPOSITION | WM_IME_ENDCOMPOSITION => {
            super::ime::message(state, message, l);
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            // SAFETY: an all-zero PAINTSTRUCT is valid writable output storage.
            let mut paint = unsafe { std::mem::zeroed() };
            // SAFETY: BeginPaint validates this child HWND's pending paint region.
            unsafe {
                BeginPaint(hwnd, &mut paint);
            }
            // SAFETY: paint is the output of the matching BeginPaint above.
            unsafe {
                EndPaint(hwnd, &paint);
            }
            if let Err(error) = super::cpu::present(state) {
                state.shared.fail(error);
            }
            state.event(ShellEvent::Redraw);
            0
        }
        WM_SIZE => {
            if state.geometry() {
                state.event(ShellEvent::Resized);
            }
            0
        }
        WM_DPICHANGED_AFTERPARENT => {
            if state.scale_source.get() == ScaleSource::System {
                state.shared.set_scale(super::dpi::scale(hwnd));
                state.event(ShellEvent::Resized);
            }
            0
        }
        WM_TIMER if w == super::native::TIMER => {
            // SAFETY: this is our own HWND-scoped one-shot timer.
            unsafe {
                KillTimer(hwnd, super::native::TIMER);
            }
            state.event(ShellEvent::Wake);
            0
        }
        super::native::WAKE => {
            state.native.wake_pending.store(false, Ordering::Release);
            state.event(ShellEvent::Wake);
            0
        }
        super::native::COMMAND => {
            state.drain();
            0
        }
        WM_GETDLGCODE => (DLGC_WANTALLKEYS | DLGC_WANTCHARS) as isize,
        // The system STATIC procedure otherwise returns HTTRANSPARENT and
        // sends real mouse input through the embedded surface to its parent.
        WM_NCHITTEST => HTCLIENT as isize,
        WM_SETCURSOR if l as u16 == HTCLIENT as u16 => {
            super::input::cursor(state.cursor.get());
            1
        }
        WM_SETFOCUS | WM_KILLFOCUS => {
            let focused = message == WM_SETFOCUS;
            if focused {
                let environment = super::window::environment(state);
                let changed = *state.environment.borrow() != environment;
                state.environment.replace(environment);
                if changed {
                    state.event(ShellEvent::EnvironmentChanged);
                }
            } else {
                super::input::cancel(state);
            }
            super::a11y::focus(state, focused);
            state.event(ShellEvent::Focused(focused));
            0
        }
        WM_DESTROY => {
            // Dispose first: the handler's close (GPU shutdown) still sees a live window.
            super::a11y::disconnect(state);
            state.dispose();
            state.shared.window_gone();
            original(state, hwnd, message, w, l)
        }
        // The tail of `wndproc` ends the child's life.
        WM_NCDESTROY => 0,
        _ => {
            // The system control class can emit double-click messages. Our
            // ClickCounter counts repeated presses itself, so retain each down.
            let input_message = match message {
                WM_LBUTTONDBLCLK => WM_LBUTTONDOWN,
                WM_RBUTTONDBLCLK => WM_RBUTTONDOWN,
                WM_MBUTTONDBLCLK => WM_MBUTTONDOWN,
                WM_XBUTTONDBLCLK => WM_XBUTTONDOWN,
                _ => message,
            };
            if let Some(result) = super::input::message(state, input_message, w, l) {
                return result;
            }
            original_if_alive(state, hwnd, message, w, l)
        }
    }
}

/// Fall through to the system procedure after App code ran; the host may have
/// destroyed the window meanwhile, and a dead HWND takes no further processing.
fn original_if_alive(state: &State, hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if state.shared.alive() {
        original(state, hwnd, message, w, l)
    } else {
        0
    }
}

fn original(state: &State, hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: attach saved the system procedure for this exact HWND before dispatch.
    unsafe { CallWindowProcW(state.original_proc.get(), hwnd, message, w, l) }
}
pub(super) fn attach(state: &Rc<State>, hwnd: HWND) -> Result<(), ShellError> {
    let raw = Rc::into_raw(state.clone());
    // SAFETY: reset only this thread's error before checking a zero-result setter.
    unsafe {
        SetLastError(0);
    }
    // SAFETY: this new STATIC's application USERDATA is unused. Store one owned
    // Rc before installing our proc; the setter does not dispatch messages.
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw as isize);
    }
    // SAFETY: read the preceding setter's thread-local error.
    if unsafe { GetLastError() } != 0 {
        // SAFETY: USERDATA was unchanged; no subclass can consume this owned Rc.
        drop(unsafe { Rc::from_raw(raw) });
        return Err(ShellError::new("SetWindowLongPtrW USERDATA failed"));
    }
    // SAFETY: replace only our child procedure on its creating UI thread.
    let old = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wndproc as *const () as isize) };
    if old == 0 {
        // SAFETY: installation failed; clear our application-owned USERDATA slot.
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        }
        // SAFETY: no procedure was installed to consume this owned Rc.
        drop(unsafe { Rc::from_raw(raw) });
        return Err(ShellError::new("SetWindowLongPtrW failed"));
    }
    // SAFETY: GWLP_WNDPROC returned the previous procedure, retained solely for
    // CallWindowProcW and restoration, as required by Win32 subclassing.
    state
        .original_proc
        .set(unsafe { std::mem::transmute::<isize, WNDPROC>(old) });
    Ok(())
}
pub(super) fn detach(state: &State, hwnd: HWND) {
    // SAFETY: restore this child's saved system proc before releasing USERDATA.
    unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWLP_WNDPROC,
            state
                .original_proc
                .get()
                .map_or(0, |proc| proc as *const () as isize),
        );
    }
    // SAFETY: detach the one Rc pointer owned by our child's USERDATA slot.
    let raw = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) } as *const State;
    if !raw.is_null() {
        // SAFETY: consume exactly the detached reference. This wndproc entry's
        // independent Rc retains State through the remaining system cleanup.
        drop(unsafe { Rc::from_raw(raw) });
    }
}
