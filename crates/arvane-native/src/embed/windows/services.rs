//! Role: Native dialog ownership and shell file drops for embedded windows.
#![allow(unsafe_code, reason = "Win32 dialog handles and owned shell drop data")]
use arvane_shell::{ShellError, ShellFileDialog};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle,
    Win32WindowHandle, WindowHandle,
};
use std::future::Future;
use std::num::NonZeroIsize;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::rc::Rc;
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub(super) struct Parent(isize);
impl Parent {
    pub(super) fn new(hwnd: isize) -> Self {
        // SAFETY: reads the root of our live child window without retaining native pointers.
        Self(unsafe { GetAncestor(hwnd, GA_ROOT) })
    }
}
impl HasWindowHandle for Parent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let hwnd = NonZeroIsize::new(self.0).ok_or(HandleError::Unavailable)?;
        // SAFETY: the root is live during this UI-thread builder access; rfd
        // copies only the opaque HWND before any dialog waiting can pump messages.
        Ok(unsafe {
            WindowHandle::borrow_raw(RawWindowHandle::Win32(Win32WindowHandle::new(hwnd)))
        })
    }
}
impl HasDisplayHandle for Parent {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::windows())
    }
}
/// Use rfd's library-owned worker so COM initialization never runs on the
/// host GUI thread. The public async builder preserves the desktop options.
fn dialog(parent: &Parent, request: &ShellFileDialog) -> Result<rfd::AsyncFileDialog, ShellError> {
    if request.title.contains('\0')
        || request
            .file_name
            .as_ref()
            .is_some_and(|name| name.contains('\0'))
        || request.filters.iter().any(|(label, extensions)| {
            label.contains('\0')
                || extensions.is_empty()
                || extensions.iter().any(|extension| {
                    extension.is_empty() || extension.contains(['\0', '.', '/', '\\'])
                })
        })
    {
        return Err(ShellError::new("invalid native file dialog options"));
    }
    let mut dialog = rfd::AsyncFileDialog::new()
        .set_parent(parent)
        .set_title(&request.title);
    if let Some(directory) = &request.directory {
        dialog = dialog.set_directory(directory);
    }
    if let Some(file_name) = &request.file_name {
        dialog = dialog.set_file_name(file_name);
    }
    for (label, extensions) in &request.filters {
        dialog = dialog.add_filter(label, extensions);
    }
    Ok(dialog)
}

fn wait<T>(future: impl Future<Output = T>) -> T {
    // rfd 0.17.2's async worker can finish its epilogue after sending the
    // result. Keep its code mapped before polling can start that worker.
    crate::module::retain_module();
    let mut future = Box::pin(future);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(result) = future.as_mut().poll(&mut context) {
            return result;
        }
        pump();
    }
}

/// One round of the modal loop that rfd's worker dialog needs from its owner
/// thread. Host windows keep painting and running timers, as with a modal
/// dialog run on this thread; State::busy queues the editor's own events.
fn pump() {
    // SAFETY: no handles are passed; the wait ends on any input or after 10 ms.
    unsafe {
        MsgWaitForMultipleObjectsEx(0, std::ptr::null(), 10, QS_ALLINPUT, MWMO_INPUTAVAILABLE)
    };
    // SAFETY: an all-zero MSG is valid writable output storage.
    let mut message: MSG = unsafe { std::mem::zeroed() };
    // SAFETY: the message buffer is valid; messages go to their own window procedures.
    while unsafe { PeekMessageW(&mut message, 0, 0, 0, PM_REMOVE) } != 0 {
        if message.message == WM_QUIT {
            // Leave quitting to the host's own loop after the dialog closes.
            // SAFETY: re-posting the host's quit request to this thread.
            unsafe { PostQuitMessage(message.wParam as i32) };
            return;
        }
        // SAFETY: the message was just removed from this thread's queue.
        unsafe { TranslateMessage(&message) };
        // SAFETY: as above; the target window procedure handles it.
        unsafe { DispatchMessageW(&message) };
    }
}

pub(super) fn pick_file(
    parent: &Parent,
    request: &ShellFileDialog,
) -> Result<Option<PathBuf>, ShellError> {
    Ok(wait(dialog(parent, request)?.pick_file()).map(|file| file.path().to_owned()))
}

pub(super) fn pick_files(
    parent: &Parent,
    request: &ShellFileDialog,
) -> Result<Option<Vec<PathBuf>>, ShellError> {
    Ok(wait(dialog(parent, request)?.pick_files()).map(|files| {
        files
            .into_iter()
            .map(|file| file.path().to_owned())
            .collect()
    }))
}

pub(super) fn save_file(
    parent: &Parent,
    request: &ShellFileDialog,
) -> Result<Option<PathBuf>, ShellError> {
    Ok(wait(dialog(parent, request)?.save_file()).map(|file| file.path().to_owned()))
}

pub(super) fn pick_folder(
    parent: &Parent,
    request: &ShellFileDialog,
) -> Result<Option<PathBuf>, ShellError> {
    Ok(wait(dialog(parent, request)?.pick_folder()).map(|file| file.path().to_owned()))
}

pub(super) fn dropped(state: &Rc<super::state::State>, drop: isize) {
    // SAFETY: WM_DROPFILES transfers this HDROP to our procedure until DragFinish.
    let count = unsafe { DragQueryFileW(drop, u32::MAX, std::ptr::null_mut(), 0) };
    let mut files = Vec::new();
    for index in 0..count {
        // SAFETY: the HDROP remains owned until DragFinish below.
        let len = unsafe { DragQueryFileW(drop, index, std::ptr::null_mut(), 0) };
        let mut text = vec![0u16; len as usize + 1];
        // SAFETY: text has room for the requested path and its terminator.
        let written = unsafe { DragQueryFileW(drop, index, text.as_mut_ptr(), text.len() as u32) };
        files.push(PathBuf::from(std::ffi::OsString::from_wide(
            &text[..written as usize],
        )));
    }
    let mut point = POINT { x: 0, y: 0 };
    // SAFETY: point is writable and the drop handle is still live.
    unsafe { DragQueryPoint(drop, &mut point) };
    // SAFETY: release exactly the HDROP transferred by this message before invoking application code.
    unsafe { DragFinish(drop) };
    let scale = state.shared.scale();
    state.event(arvane_shell::ShellEvent::File {
        phase: arvane_shell::FilePhase::Drop,
        files,
        position: kivact_types::Point {
            x: point.x as f32 / scale,
            y: point.y as f32 / scale,
        },
    });
}
