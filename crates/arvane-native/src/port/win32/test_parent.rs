//! Role: Real Win32 host window for embedding end-to-end tests.
use arvane_shell::ShellError;
use kivact_types::ParentWindow;
use kivact_types::render::CpuFrame;
use kivact_types::{KeyCode, MouseButton};
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BeginPaint, BitBlt, ClientToScreen, CreateCompatibleDC,
    CreateDIBSection, CreateSolidBrush, DIB_RGB_COLORS, DeleteDC, DeleteObject, EndPaint, FillRect,
    GetDC, HBITMAP, HDC, HGDIOBJ, ReleaseDC, SRCCOPY, SelectObject,
};
use windows_sys::Win32::Storage::Xps::PrintWindow;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleHandleExW,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    MAPVK_VK_TO_VSC_EX, MapVirtualKeyW, VK_BACK, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1,
    VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12, VK_HOME,
    VK_LEFT, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, FindWindowExW, GWLP_HINSTANCE, GWLP_ID, GWLP_USERDATA, GetClientRect,
    GetWindowLongPtrW, GetWindowRect, HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST, IsWindow, PM_REMOVE,
    PW_RENDERFULLCONTENT, PeekMessageW, PostMessageW, RegisterClassW, SC_CLOSE, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SetWindowLongPtrW, SetWindowPos, TranslateMessage, UnregisterClassW,
    WM_CHAR, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SYSCOMMAND, WM_XBUTTONDOWN, WM_XBUTTONUP, WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS, WS_OVERLAPPEDWINDOW, WS_VISIBLE, XBUTTON1, XBUTTON2,
};
const NAME: &[u16] = &[
    65, 114, 118, 97, 110, 101, 69, 109, 98, 101, 100, 84, 101, 115, 116, 80, 97, 114, 101, 110,
    116, 0,
];
static CLASS: Mutex<(usize, isize)> = Mutex::new((0, 0));
/// A real top-level host. Use and drop it on the thread that opened it.
pub struct TestParent {
    point: Cell<(f64, f64)>,
    buttons: Cell<usize>,
    hwnd: HWND,
    owner: std::thread::ThreadId,
    _ui: PhantomData<Rc<()>>,
}
/// The window procedure of the test host and its cover windows.
///
/// # Safety
///
/// Only Windows may call this, as the window procedure of a window of this
/// module's classes, on the thread that created `hwnd`, with the arguments
/// of the message it is delivering. The window's `GWLP_USERDATA` slot is an
/// integer counter and `GWLP_ID` a color; neither may hold a pointer.
unsafe extern "system" fn parent_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_SYSCOMMAND && w & 0xfff0 == SC_CLOSE as usize {
        // SAFETY: this test host uses its own USERDATA as an integer counter.
        let count = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
        // SAFETY: store an integer, never an owning pointer, on this test host.
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, count + 1);
        }
        return 0;
    }
    // SAFETY: read only this test window's integer control identifier.
    let color = unsafe { GetWindowLongPtrW(hwnd, GWLP_ID) };
    if msg == WM_PAINT && color != 0 {
        // SAFETY: zeroed paint storage is valid writable output.
        let mut paint = unsafe { std::mem::zeroed() };
        // SAFETY: validate this cover's update region and borrow its paint DC.
        let dc = unsafe { BeginPaint(hwnd, &mut paint) };
        // SAFETY: a COLORREF creates an owned brush without borrowing memory.
        let brush = unsafe { CreateSolidBrush((color - 1) as u32) };
        // SAFETY: paint's rectangle and DC belong to the same cover window.
        unsafe {
            FillRect(dc, &paint.rcPaint, brush);
        }
        // SAFETY: the temporary brush is no longer used by FillRect.
        unsafe {
            DeleteObject(brush);
        }
        // SAFETY: pair this paint structure with its BeginPaint call.
        unsafe {
            EndPaint(hwnd, &paint);
        }
        return 0;
    }
    // SAFETY: remaining messages use the standard test window procedure.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}
impl TestParent {
    /// Create a host-owned child container at physical coordinates.
    /// The outer host destroys it when it closes.
    pub fn parent_in(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<ParentWindow, ShellError> {
        assert_eq!(self.owner, std::thread::current().id());
        if width == 0 || height == 0 || [x, y, width, height].iter().any(|v| *v > i32::MAX as u32) {
            return Err(ShellError::new("invalid test parent container coordinates"));
        }
        let class = [83u16, 84, 65, 84, 73, 67, 0];
        // SAFETY: STATIC is a system class, the name is terminated and the parent
        // belongs to this GUI thread; Windows owns the child until parent destruction.
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                x as i32,
                y as i32,
                width as i32,
                height as i32,
                self.hwnd,
                0,
                0,
                std::ptr::null(),
            )
        };
        std::num::NonZeroIsize::new(hwnd)
            .map(ParentWindow::Win32)
            .ok_or_else(|| ShellError::new("test parent container creation failed"))
    }
    /// Open a visible host with the requested physical client extent.
    pub fn open(width: u32, height: u32) -> Result<Self, ShellError> {
        if width == 0 || height == 0 || width > i32::MAX as u32 || height > i32::MAX as u32 {
            return Err(ShellError::new(
                "test parent dimensions must be positive Win32 coordinates",
            ));
        }
        let mut class = CLASS
            .lock()
            .expect("test parent class lock poisoned: a panic already happened");
        register_parent_class(&mut class)?;
        let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN;
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        // SAFETY: rect is writable and contains the requested physical client extent.
        unsafe {
            AdjustWindowRectEx(&mut rect, style, 0, 0);
        }
        // SAFETY: class belongs to this DLL, all pointer arguments remain live, and
        // the returned top-level HWND is confined to this test host's GUI thread.
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                NAME.as_ptr(),
                NAME.as_ptr(),
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                0,
                0,
                class.1,
                std::ptr::null(),
            )
        };
        if hwnd == 0 {
            if class.0 == 0 {
                // SAFETY: creation failed and this registration has no live HWNDs.
                unsafe {
                    UnregisterClassW(NAME.as_ptr(), class.1);
                }
                class.1 = 0;
            }
            return Err(ShellError::new("test parent creation failed"));
        }
        class.0 += 1;
        Ok(Self {
            point: Cell::new((0.0, 0.0)),
            buttons: Cell::new(0),
            hwnd,
            owner: std::thread::current().id(),
            _ui: PhantomData,
        })
    }
    /// Portable parent handle to pass to `embed::open`.
    pub fn parent(&self) -> ParentWindow {
        ParentWindow::Win32(std::num::NonZeroIsize::new(self.hwnd).expect("live native handle"))
    }
    /// Raw HWND for plugin wrappers.
    pub fn win32_hwnd(&self) -> Option<*mut std::ffi::c_void> {
        Some(self.hwnd as *mut std::ffi::c_void)
    }
    /// X11 handles are unavailable on Windows.
    pub fn x11_window(&self) -> Option<u32> {
        None
    }
    /// Pump the host thread's messages for the given duration.
    pub fn pump(&self, duration: Duration) {
        assert_eq!(
            self.owner,
            std::thread::current().id(),
            "test host must stay on its GUI thread"
        );
        let deadline = Instant::now() + duration;
        loop {
            // SAFETY: zeroed MSG is valid writable message output storage.
            let mut msg = unsafe { std::mem::zeroed() };
            loop {
                // SAFETY: this is the test host event loop, authorized to pump every
                // window owned by this GUI thread, unlike the production child loop.
                if unsafe { PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) } == 0 {
                    break;
                }
                if msg.message != WM_QUIT {
                    // SAFETY: msg is the initialized message returned by PeekMessageW.
                    unsafe {
                        TranslateMessage(&msg);
                    }
                    // SAFETY: dispatch runs the message's owning GUI-thread window proc.
                    unsafe {
                        DispatchMessageW(&msg);
                    }
                }
            }
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    fn child(&self) -> Result<HWND, ShellError> {
        let mut child = 0;
        loop {
            // SAFETY: search only direct children of this test host.
            child = unsafe { FindWindowExW(self.hwnd, child, std::ptr::null(), std::ptr::null()) };
            if child == 0 {
                return Err(ShellError::new("the test parent has no embedded child"));
            }
            // SAFETY: this read-only query returns the child's USERDATA slot, which
            // only our attach sets (to its State) and only our detach clears. The
            // window procedure cannot identify it: a GPU swapchain subclasses it.
            if unsafe { GetWindowLongPtrW(child, GWLP_USERDATA) } != 0 {
                return Ok(child);
            }
        }
    }

    fn post(&self, msg: u32, w: WPARAM, l: LPARAM) -> Result<(), ShellError> {
        let child = self.child()?;
        // SAFETY: input is queued to the test child only, never another application.
        if unsafe { PostMessageW(child, msg, w, l) } == 0 {
            Err(ShellError::new("posting test input failed"))
        } else {
            Ok(())
        }
    }
    /// Queue a button press in physical child-client coordinates.
    pub fn press_button(&self, button: MouseButton, x: f64, y: f64) -> Result<(), ShellError> {
        self.button(button, x, y, true)
    }
    /// Queue a button release in physical child-client coordinates.
    pub fn release_button(&self, button: MouseButton, x: f64, y: f64) -> Result<(), ShellError> {
        self.button(button, x, y, false)
    }
    fn button(&self, button: MouseButton, x: f64, y: f64, down: bool) -> Result<(), ShellError> {
        use windows_sys::Win32::System::SystemServices::{
            MK_LBUTTON, MK_MBUTTON, MK_RBUTTON, MK_XBUTTON1, MK_XBUTTON2,
        };
        let (press, release, mask, high) = match button {
            MouseButton::Primary => (WM_LBUTTONDOWN, WM_LBUTTONUP, MK_LBUTTON, 0),
            MouseButton::Secondary => (WM_RBUTTONDOWN, WM_RBUTTONUP, MK_RBUTTON, 0),
            MouseButton::Middle => (WM_MBUTTONDOWN, WM_MBUTTONUP, MK_MBUTTON, 0),
            MouseButton::Back => (WM_XBUTTONDOWN, WM_XBUTTONUP, MK_XBUTTON1, XBUTTON1),
            MouseButton::Forward => (WM_XBUTTONDOWN, WM_XBUTTONUP, MK_XBUTTON2, XBUTTON2),
        };
        let held = if down {
            self.buttons.get() | mask as usize
        } else {
            self.buttons.get() & !(mask as usize)
        };
        self.post(
            if down { press } else { release },
            held | ((high as usize) << 16),
            coordinates(x, y),
        )?;
        self.buttons.set(held);
        self.point.set((x, y));
        Ok(())
    }
    /// Queue a primary press in physical child-client coordinates.
    pub fn press(&self, x: f64, y: f64) -> Result<(), ShellError> {
        self.press_button(MouseButton::Primary, x, y)
    }
    /// Queue pointer motion in physical child-client coordinates.
    pub fn move_to(&self, x: f64, y: f64) -> Result<(), ShellError> {
        self.post(WM_MOUSEMOVE, self.buttons.get(), coordinates(x, y))?;
        self.point.set((x, y));
        Ok(())
    }
    /// Queue a primary release in physical child-client coordinates.
    pub fn release(&self, x: f64, y: f64) -> Result<(), ShellError> {
        self.release_button(MouseButton::Primary, x, y)
    }
    /// Queue a complete primary click.
    pub fn click(&self, x: f64, y: f64) -> Result<(), ShellError> {
        self.press(x, y)?;
        self.release(x, y)
    }
    /// Queue wheel notches at the most recently injected mouse point.
    /// Positive values scroll right/down; units are native wheel notches.
    pub fn wheel(&self, lines_x: f64, lines_y: f64) -> Result<(), ShellError> {
        let child = self.child()?;
        let (x, y) = self.point.get();
        let mut point = POINT {
            x: x.round() as i32,
            y: y.round() as i32,
        };
        // SAFETY: this live test child owns the client coordinates being converted.
        if unsafe { ClientToScreen(child, &mut point) } == 0 {
            return Err(ShellError::new("converting test wheel coordinates failed"));
        }
        for (message, count) in [(WM_MOUSEHWHEEL, lines_x), (WM_MOUSEWHEEL, -lines_y)] {
            let delta = count * 120.0;
            if !delta.is_finite()
                || delta < i16::MIN as f64
                || delta > i16::MAX as f64
                || delta.fract() != 0.0
            {
                return Err(ShellError::unsupported(
                    "wheel delta is outside native Win32 precision",
                ));
            }
            if delta != 0.0 {
                self.post(
                    message,
                    ((delta as i16 as u16 as u32) << 16) as usize,
                    coordinates(point.x as f64, point.y as f64),
                )?;
            }
        }
        Ok(())
    }
    /// Queue one physical key press and release, using the current keyboard layout.
    pub fn key(&self, code: KeyCode) -> Result<(), ShellError> {
        let vk = virtual_key(code)
            .ok_or_else(|| ShellError::unsupported("this test key has no Win32 mapping"))?;
        // SAFETY: the current layout maps a virtual key to an OS scan code.
        let scan = unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC_EX) };
        let bits = ((scan & 0xff) << 16) | if scan & 0xff00 != 0 { 1 << 24 } else { 0 } | 1;
        self.post(WM_KEYDOWN, vk as usize, bits as isize)?;
        self.post(
            WM_KEYUP,
            vk as usize,
            (bits | (1 << 30) | (1 << 31)) as isize,
        )
    }
    /// Queue UTF-16 character messages, as a host TranslateMessage would.
    pub fn text(&self, text: &str) -> Result<(), ShellError> {
        for unit in text.encode_utf16() {
            self.post(WM_CHAR, unit as usize, 1)?;
        }
        Ok(())
    }
    /// Destroy the host (and its embedded child) synchronously.
    pub fn close(&self) {
        if !self.is_closed() {
            // SAFETY: TestParent cannot leave its creating thread, and owns its HWND.
            unsafe {
                DestroyWindow(self.hwnd);
            }
        }
    }
    /// Whether the user or test has closed this host.
    pub fn is_closed(&self) -> bool {
        // SAFETY: IsWindow checks an opaque value without dereferencing it.
        unsafe { IsWindow(self.hwnd) == 0 }
    }
    /// Count SC_CLOSE requests delivered to the host without closing it.
    pub fn close_requests(&self) -> u32 {
        // SAFETY: this test host stores an integer counter in its own USERDATA.
        unsafe { GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as u32 }
    }
    /// Place a colored host-owned sibling above the editor or its container.
    pub fn cover(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        rgb: [u8; 3],
    ) -> Result<(), ShellError> {
        if width == 0 || height == 0 || [x, y, width, height].iter().any(|v| *v > i32::MAX as u32) {
            return Err(ShellError::new("invalid test cover coordinates"));
        }
        let color = u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16);
        // SAFETY: read the module used to create this live host's local class.
        let instance = unsafe { GetWindowLongPtrW(self.hwnd, GWLP_HINSTANCE) };
        // SAFETY: reuse the registered test class; the host owns this child.
        let cover = unsafe {
            CreateWindowExW(
                0,
                NAME.as_ptr(),
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
                x as i32,
                y as i32,
                width as i32,
                height as i32,
                self.hwnd,
                (color + 1) as isize,
                instance,
                std::ptr::null(),
            )
        };
        if cover == 0 {
            return Err(ShellError::new("test cover creation failed"));
        }
        // SAFETY: move this host-owned cover to the top of its sibling chain.
        unsafe {
            SetWindowPos(
                cover,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
        Ok(())
    }
    /// Read host client pixels from the screen, including overlapping siblings.
    /// Raise only this owned test host without changing keyboard activation.
    pub fn capture_host(&self) -> Result<CpuFrame, ShellError> {
        // SAFETY: this GUI thread owns the host; only its z-order changes.
        if unsafe {
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            )
        } == 0
        {
            return Err(ShellError::new(
                "raising the test host for screen capture failed",
            ));
        }
        // SAFETY: wait for pending composition without changing process settings.
        if unsafe { windows_sys::Win32::Graphics::Dwm::DwmFlush() } < 0 {
            return Err(ShellError::new("synchronizing test screen capture failed"));
        }
        let frame = capture(self.hwnd, false, true);
        // SAFETY: as above; return the owned test host to the normal band.
        unsafe {
            SetWindowPos(
                self.hwnd,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            );
        }
        frame
    }
    /// Read child pixels. Black child-DC results retry the top-level PrintWindow
    /// path and crop the child's screen rectangle from the host image.
    pub fn capture(&self) -> Result<CpuFrame, ShellError> {
        let child = self.child()?;
        let frame = capture(child, false, false)?;
        if frame
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[..3] != [0, 0, 0])
        {
            return Ok(frame);
        }
        let parent = capture(self.hwnd, true, false)?;
        let mut point = POINT { x: 0, y: 0 };
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: child is live and point is valid writable coordinate storage.
        unsafe {
            ClientToScreen(child, &mut point);
        }
        // SAFETY: this host HWND is live and rect is writable outer-bounds storage.
        unsafe {
            GetWindowRect(self.hwnd, &mut rect);
        }
        let (x, y) = (point.x - rect.left, point.y - rect.top);
        if x < 0
            || y < 0
            || x as u32 + frame.width > parent.width
            || y as u32 + frame.height > parent.height
        {
            return Err(ShellError::new("test child lies outside the captured host"));
        }
        let mut rgba = Vec::with_capacity(frame.rgba.len());
        for row in 0..frame.height {
            let start = (((y as u32 + row) * parent.width + x as u32) * 4) as usize;
            rgba.extend_from_slice(&parent.rgba[start..start + frame.width as usize * 4]);
        }
        Ok(CpuFrame {
            width: frame.width,
            height: frame.height,
            rgba,
        })
    }
}
impl Drop for TestParent {
    fn drop(&mut self) {
        self.close();
        let mut class = CLASS
            .lock()
            .expect("test parent class lock poisoned: a panic already happened");
        class.0 -= 1;
        if class.0 == 0 {
            // SAFETY: the final host HWND was destroyed before unregistering its DLL class.
            unsafe {
                UnregisterClassW(NAME.as_ptr(), class.1);
            }
            class.1 = 0;
        }
    }
}
fn coordinates(x: f64, y: f64) -> isize {
    ((x.round() as i16 as u16 as u32) | ((y.round() as i16 as u16 as u32) << 16)) as isize
}
fn virtual_key(code: KeyCode) -> Option<u16> {
    use KeyCode::*;
    Some(match code {
        KeyA => 65,
        KeyB => 66,
        KeyC => 67,
        KeyD => 68,
        KeyE => 69,
        KeyF => 70,
        KeyG => 71,
        KeyH => 72,
        KeyI => 73,
        KeyJ => 74,
        KeyK => 75,
        KeyL => 76,
        KeyM => 77,
        KeyN => 78,
        KeyO => 79,
        KeyP => 80,
        KeyQ => 81,
        KeyR => 82,
        KeyS => 83,
        KeyT => 84,
        KeyU => 85,
        KeyV => 86,
        KeyW => 87,
        KeyX => 88,
        KeyY => 89,
        KeyZ => 90,
        Digit0 => 48,
        Digit1 => 49,
        Digit2 => 50,
        Digit3 => 51,
        Digit4 => 52,
        Digit5 => 53,
        Digit6 => 54,
        Digit7 => 55,
        Digit8 => 56,
        Digit9 => 57,
        Enter => VK_RETURN,
        Tab => VK_TAB,
        Space => VK_SPACE,
        Backspace => VK_BACK,
        Delete => VK_DELETE,
        Escape => VK_ESCAPE,
        ArrowUp => VK_UP,
        ArrowDown => VK_DOWN,
        ArrowLeft => VK_LEFT,
        ArrowRight => VK_RIGHT,
        Home => VK_HOME,
        End => VK_END,
        PageUp => VK_PRIOR,
        PageDown => VK_NEXT,
        F1 => VK_F1,
        F2 => VK_F2,
        F3 => VK_F3,
        F4 => VK_F4,
        F5 => VK_F5,
        F6 => VK_F6,
        F7 => VK_F7,
        F8 => VK_F8,
        F9 => VK_F9,
        F10 => VK_F10,
        F11 => VK_F11,
        F12 => VK_F12,
        _ => return None,
    })
}
struct CaptureResources {
    hwnd: HWND,
    dc: HDC,
    memory: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
}
impl Drop for CaptureResources {
    fn drop(&mut self) {
        if self.old != 0 {
            // SAFETY: restore the object borrowed from this memory DC before deleting its bitmap.
            unsafe {
                SelectObject(self.memory, self.old);
            }
        }
        if self.bitmap != 0 {
            // SAFETY: this owned DIB has been deselected from the memory DC.
            unsafe {
                DeleteObject(self.bitmap);
            }
        }
        if self.memory != 0 {
            // SAFETY: this memory DC was created with CreateCompatibleDC in capture.
            unsafe {
                DeleteDC(self.memory);
            }
        }
        if self.dc != 0 {
            // SAFETY: this DC was borrowed with GetDC from the same HWND.
            unsafe {
                ReleaseDC(self.hwnd, self.dc);
            }
        }
    }
}
/// Register the host class before creating its first window.
fn register_parent_class(class: &mut (usize, isize)) -> Result<(), ShellError> {
    if class.0 == 0 {
        let mut module = 0;
        // SAFETY: the proc's address identifies this DLL without retaining it.
        let found = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                parent_proc as *const () as *const u16,
                &mut module,
            )
        };
        if found == 0 {
            return Err(ShellError::new("test parent module lookup failed"));
        }
        let definition = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(parent_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: module,
            hIcon: 0,
            hCursor: 0,
            hbrBackground: 0,
            lpszMenuName: std::ptr::null(),
            lpszClassName: NAME.as_ptr(),
        };
        // SAFETY: definition and its class name live through registration.
        if unsafe { RegisterClassW(&definition) } == 0 {
            return Err(ShellError::new("test parent class registration failed"));
        }
        class.1 = module;
    }
    Ok(())
}

fn capture(hwnd: HWND, print: bool, screen: bool) -> Result<CpuFrame, ShellError> {
    let (width, height) = capture_extent(hwnd, print)?;
    let bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| ShellError::new("test capture dimensions overflow"))?;
    let mut origin = POINT { x: 0, y: 0 };
    if screen {
        // SAFETY: convert the live host's client origin to screen coordinates.
        if unsafe { ClientToScreen(hwnd, &mut origin) } == 0 {
            return Err(ShellError::new("test host coordinate conversion failed"));
        }
    }
    let mut resources = CaptureResources {
        hwnd: if screen { 0 } else { hwnd },
        dc: 0,
        memory: 0,
        bitmap: 0,
        old: 0,
    };
    // SAFETY: GetDC borrows this live child or host HWND's device context.
    resources.dc = unsafe { GetDC(resources.hwnd) };
    if resources.dc == 0 {
        return Err(ShellError::new("GetDC failed"));
    }
    // SAFETY: create a compatible memory DC from the borrowed window DC.
    resources.memory = unsafe { CreateCompatibleDC(resources.dc) };
    if resources.memory == 0 {
        return Err(ShellError::new("CreateCompatibleDC failed"));
    }
    // SAFETY: zeroed BITMAPINFO is valid storage before setting its DIB header fields.
    let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = width;
    info.bmiHeader.biHeight = -height;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    let mut bits = std::ptr::null_mut();
    // SAFETY: a size-tagged top-down 32-bit DIB header and writable bits output
    // create an owned bitmap whose bytes remain valid until resource cleanup.
    resources.bitmap =
        unsafe { CreateDIBSection(resources.dc, &info, DIB_RGB_COLORS, &mut bits, 0, 0) };
    if resources.bitmap == 0 || bits.is_null() {
        return Err(ShellError::new("CreateDIBSection failed"));
    }
    // SAFETY: select this owned bitmap into the memory DC, retaining the old object.
    resources.old = unsafe { SelectObject(resources.memory, resources.bitmap) };
    if resources.old == 0 || resources.old == -1 {
        resources.old = 0;
        return Err(ShellError::new("SelectObject failed"));
    }
    let result = if print {
        // SAFETY: host rendering targets our bitmap-backed memory DC synchronously.
        unsafe { PrintWindow(hwnd, resources.memory, PW_RENDERFULLCONTENT) }
    } else {
        // SAFETY: the source window DC and destination bitmap have the checked extent.
        unsafe {
            BitBlt(
                resources.memory,
                0,
                0,
                width,
                height,
                resources.dc,
                origin.x,
                origin.y,
                SRCCOPY,
            )
        }
    };
    if result == 0 {
        return Err(ShellError::new("native test capture failed"));
    }
    // SAFETY: the created 32-bit DIB owns exactly width*height*4 initialized bytes
    // after the successful copy. It remains selected and live through this copy.
    let pixels = unsafe { std::slice::from_raw_parts(bits.cast::<u8>(), bytes) };
    let mut rgba = pixels.to_vec();
    for pixel in rgba.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
        pixel[3] = 255;
    }
    Ok(CpuFrame {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

/// Validate the native rectangle before allocating a capture DIB.
fn capture_extent(hwnd: HWND, print: bool) -> Result<(i32, i32), ShellError> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if print {
        // SAFETY: rect is writable output storage for this live host's outer bounds.
        unsafe {
            GetWindowRect(hwnd, &mut rect);
        }
    } else {
        // SAFETY: rect is writable output storage for this live child's client bounds.
        unsafe {
            GetClientRect(hwnd, &mut rect);
        }
    }
    let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
    if width <= 0 || height <= 0 {
        return Err(ShellError::new("test capture has an empty rectangle"));
    }
    Ok((width, height))
}
