// IMM32 context, composition and candidate handling adapted with reference to
// winit 0.30.13, src/platform_impl/windows/ime.rs, Apache-2.0.
// Licensed under the Apache License, Version 2.0; see LICENSE-APACHE.
// Changes: child-only ownership, initialized buffers and UTF-16 cursor conversion.
//! Role: Child-window IMM32 composition and candidate placement.
//! IMM32 coordinates and offsets are physical pixels and UTF-16; shell
//! preedit selections count UTF-8 bytes. No process input settings change.
#![allow(unsafe_code, reason = "IMM32 is the native Windows text input API")]
use super::state::State;
use arvane_shell::ShellEvent;
use std::rc::Rc;
use windows_sys::Win32::Foundation::{POINT, RECT};
use windows_sys::Win32::UI::Input::Ime::{
    CANDIDATEFORM, CFS_EXCLUDE, CFS_POINT, COMPOSITIONFORM, GCS_COMPATTR, GCS_COMPSTR,
    GCS_CURSORPOS, GCS_RESULTSTR, IACE_DEFAULT, ImmAssociateContextEx, ImmGetCompositionStringW,
    ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow, ImmSetCompositionWindow,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION};

struct Context {
    hwnd: isize,
    handle: isize,
}
impl Context {
    fn new(hwnd: isize) -> Self {
        // SAFETY: the live child is accessed on its creating UI thread.
        let handle = unsafe { ImmGetContext(hwnd) };
        Self { hwnd, handle }
    }
    fn text(&self, kind: u32) -> Option<Vec<u16>> {
        if self.handle == 0 {
            return None;
        }
        // SAFETY: null output queries the composition buffer length only.
        let bytes = unsafe { ImmGetCompositionStringW(self.handle, kind, std::ptr::null_mut(), 0) };
        if bytes < 0 || bytes % 2 != 0 {
            return None;
        }
        let mut text = vec![0u16; bytes as usize / 2];
        // SAFETY: initialized UTF-16 storage covers exactly bytes writable bytes.
        let written = unsafe {
            ImmGetCompositionStringW(self.handle, kind, text.as_mut_ptr().cast(), bytes as u32)
        };
        if written < 0 || written > bytes || written % 2 != 0 {
            return None;
        }
        text.truncate(written as usize / 2);
        Some(text)
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        if self.handle != 0 {
            // SAFETY: release only the context acquired by this guard for the same HWND.
            unsafe { ImmReleaseContext(self.hwnd, self.handle) };
        }
    }
}
pub(super) fn allowed(state: &State, allowed: bool) {
    // SAFETY: association affects only our child HWND on the owning UI thread.
    unsafe {
        ImmAssociateContextEx(
            state.native.hwnd(),
            0,
            if allowed { IACE_DEFAULT } else { 0 },
        )
    };
}
pub(super) fn area(state: &State, area: kivact_types::Rect) {
    let context = Context::new(state.native.hwnd());
    if context.handle == 0 {
        return;
    }
    let scale = state.shared.scale();
    let x = (area.x * scale).round() as i32;
    let y = (area.y * scale).round() as i32;
    let bottom = ((area.y + area.height) * scale).round() as i32;
    let rect = RECT {
        left: x,
        top: y,
        right: ((area.x + area.width) * scale).round() as i32,
        bottom,
    };
    let candidate = CANDIDATEFORM {
        dwIndex: 0,
        dwStyle: CFS_EXCLUDE,
        ptCurrentPos: POINT { x, y },
        rcArea: rect,
    };
    let composition = COMPOSITIONFORM {
        dwStyle: CFS_POINT,
        ptCurrentPos: POINT { x, y: bottom },
        rcArea: rect,
    };
    // SAFETY: context is acquired on this thread; candidate is initialized native input storage.
    unsafe { ImmSetCandidateWindow(context.handle, &candidate) };
    // SAFETY: context remains live and composition is initialized native input storage.
    unsafe { ImmSetCompositionWindow(context.handle, &composition) };
}
pub(super) fn message(state: &Rc<State>, message: u32, flags: isize) {
    if message == WM_IME_STARTCOMPOSITION {
        state.event(ShellEvent::Preedit {
            text: String::new(),
            selection: None,
        });
        return;
    }
    if message == WM_IME_ENDCOMPOSITION {
        state.event(ShellEvent::ImeDisabled);
        return;
    }
    let mut events = Vec::new();
    {
        let context = Context::new(state.native.hwnd());
        if flags as u32 & GCS_RESULTSTR != 0
            && let Some(text) = context.text(GCS_RESULTSTR)
        {
            events.push(ShellEvent::Preedit {
                text: String::new(),
                selection: None,
            });
            events.push(ShellEvent::Commit(String::from_utf16_lossy(&text)));
        }
        if flags as u32 & (GCS_COMPSTR | GCS_CURSORPOS | GCS_COMPATTR) != 0
            && let Some(text) = context.text(GCS_COMPSTR)
        {
            // SAFETY: GCS_CURSORPOS returns a scalar UTF-16 offset and writes no buffer.
            let cursor = unsafe {
                ImmGetCompositionStringW(context.handle, GCS_CURSORPOS, std::ptr::null_mut(), 0)
            };
            let selection = usize::try_from(cursor)
                .ok()
                .filter(|at| *at <= text.len())
                .and_then(|at| String::from_utf16(&text[..at]).ok())
                .map(|prefix| (prefix.len(), prefix.len()));
            events.push(ShellEvent::Preedit {
                text: String::from_utf16_lossy(&text),
                selection,
            });
        }
    }
    for event in events {
        state.event(event);
    }
}
