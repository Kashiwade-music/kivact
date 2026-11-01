//! Role: CPU present through a cache DC clipped below higher siblings.
use super::state::State;
use arvane_shell::ShellError;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;

pub(super) fn present(state: &State) -> Result<(), ShellError> {
    let (width, height) = state.pixel_size.get();
    if width == 0 || height == 0 || !state.cpu_shown.get() {
        return Ok(());
    }
    let hwnd = state.native.hwnd();
    // SAFETY: borrow a private cached DC for this live child on its GUI thread.
    let dc = unsafe { GetDCEx(hwnd, 0, DCX_CACHE | DCX_CLIPSIBLINGS) };
    if dc == 0 {
        return Err(ShellError::new("GetDCEx failed"));
    }
    let mut clip = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    // SAFETY: clip is writable output for this borrowed DC's visible bounds.
    let region = unsafe { GetClipBox(dc, &mut clip) };
    if region == NULLREGION || region == ERROR {
        // SAFETY: release the exact DC borrowed above from the same child.
        unsafe {
            ReleaseDC(hwnd, dc);
        }
        return if region == NULLREGION {
            Ok(())
        } else {
            Err(ShellError::new("GetClipBox failed"))
        };
    }
    // SAFETY: zero is valid initial storage for a size-tagged bitmap header.
    let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = width as i32;
    info.bmiHeader.biHeight = -(height as i32);
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    let pixels = state.pixels.borrow();
    // SAFETY: pixels holds width*height BGRA words for the top-down 32-bit DIB.
    let copied = unsafe {
        SetDIBitsToDevice(
            dc,
            0,
            0,
            width,
            height,
            0,
            0,
            0,
            height,
            pixels.as_ptr().cast(),
            &info,
            DIB_RGB_COLORS,
        )
    };
    // SAFETY: release the exact DC borrowed above from the same child.
    unsafe {
        ReleaseDC(hwnd, dc);
    }
    if copied == 0 {
        Err(ShellError::new("SetDIBitsToDevice failed"))
    } else {
        Ok(())
    }
}
