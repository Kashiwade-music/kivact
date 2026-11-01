//! Role: Raw presentation target; retaining it never destroys a host window.
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};
use std::num::NonZeroIsize;
use std::sync::Arc;
pub(super) struct Target {
    pub(super) instance: isize,
    pub(super) native: Arc<super::native::Native>,
    pub(super) shared: Arc<crate::embed::child::Shared>,
}
impl HasWindowHandle for Target {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if !self.shared.alive.load(std::sync::atomic::Ordering::Acquire) {
            return Err(HandleError::Unavailable);
        }
        let mut handle = Win32WindowHandle::new(
            NonZeroIsize::new(self.native.hwnd()).ok_or(HandleError::Unavailable)?,
        );
        handle.hinstance = NonZeroIsize::new(self.instance);
        // SAFETY: target use is synchronous on the owning UI thread. Shared alive
        // rejects access after destruction; the caller retains target for surface use.
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(handle)) })
    }
}
impl HasDisplayHandle for Target {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // SAFETY: Windows display handles carry no borrowed pointer.
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Windows(WindowsDisplayHandle::new()))
        })
    }
}
