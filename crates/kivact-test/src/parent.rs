//! Role: Public native parent-window driver without backend types.
use super::Screenshot;
use kivact_core::embed::Parent;
use kivact_core::view::KeyCode;
use kivact_core::{Result, UiError};
use kivact_runtime::ErrorKind;
use std::ffi::c_void;
use std::time::Duration;

/// A real host for testing `App::embed`. Keep it on its creating thread and
/// pump messages while waiting for rendering or cross-thread operations.
/// Coordinates and capture dimensions use physical pixels.
pub struct TestParent(kivact_engine::testing::TestParent);
fn error(error: impl std::fmt::Display) -> UiError {
    UiError::new(ErrorKind::Platform, error.to_string(), None)
}
impl TestParent {
    /// Open a real parent with the given physical size.
    pub fn open(width: u32, height: u32) -> Result<Self> {
        kivact_engine::testing::TestParent::open(width, height)
            .map(Self)
            .map_err(error)
    }
    /// Create a host-owned container at physical coordinates. It stays alive
    /// until the outer host closes; use one per editor for side-by-side tests.
    pub fn parent_in(&self, x: u32, y: u32, width: u32, height: u32) -> Result<Parent> {
        self.0
            .parent_in(x, y, width, height)
            .map(|parent| match parent {
                kivact_types::ParentWindow::Win32(hwnd) => Parent::win32(hwnd.get() as *mut c_void),
                kivact_types::ParentWindow::X11(id) => Parent::x11(id.get()),
            })
            .map_err(error)
    }
    /// The host window to pass to `App::embed`.
    pub fn parent(&self) -> Parent {
        if let Some(hwnd) = self.win32_hwnd() {
            Parent::win32(hwnd)
        } else {
            Parent::x11(self.x11_window().expect("native test parent has a window"))
        }
    }
    /// Raw Windows handle for plugin wrapper tests.
    pub fn win32_hwnd(&self) -> Option<*mut c_void> {
        self.0.win32_hwnd()
    }
    /// Raw X11 id for plugin wrapper tests.
    pub fn x11_window(&self) -> Option<u32> {
        self.0.x11_window()
    }
    /// Dispatch native messages for at most this duration.
    pub fn pump(&self, duration: Duration) {
        self.0.pump(duration);
    }
    /// Send a native click.
    pub fn click(&self, x: f64, y: f64) -> Result<()> {
        self.0.click(x, y).map_err(error)
    }
    /// Send a native pointer press.
    pub fn press(&self, x: f64, y: f64) -> Result<()> {
        self.0.press(x, y).map_err(error)
    }
    /// Send native pointer movement.
    pub fn move_to(&self, x: f64, y: f64) -> Result<()> {
        self.0.move_to(x, y).map_err(error)
    }
    /// Send a native pointer release.
    pub fn release(&self, x: f64, y: f64) -> Result<()> {
        self.0.release(x, y).map_err(error)
    }
    /// Send native wheel notches at the most recently injected mouse point.
    /// Positive values scroll right/down.
    pub fn wheel(&self, lines_x: f64, lines_y: f64) -> Result<()> {
        self.0.wheel(lines_x, lines_y).map_err(error)
    }
    /// Send a native key.
    pub fn key(&self, key: KeyCode) -> Result<()> {
        self.0.key(key).map_err(error)
    }
    /// Send native text input.
    pub fn text(&self, text: &str) -> Result<()> {
        self.0.text(text).map_err(error)
    }
    /// Read the child window's presented pixels from the OS.
    pub fn capture(&self) -> Result<Screenshot> {
        let frame = self.0.capture().map_err(error)?;
        Ok(Screenshot {
            width: frame.width,
            height: frame.height,
            pixels: frame.to_softbuffer(),
        })
    }
    /// Place a colored sibling over the editor at physical host coordinates.
    pub fn cover(&self, x: u32, y: u32, width: u32, height: u32, rgb: [u8; 3]) -> Result<()> {
        self.0.cover(x, y, width, height, rgb).map_err(error)
    }
    /// Read the host client area from the screen, including overlapping siblings.
    pub fn capture_host(&self) -> Result<Screenshot> {
        let frame = self.0.capture_host().map_err(error)?;
        Ok(Screenshot {
            width: frame.width,
            height: frame.height,
            pixels: frame.to_softbuffer(),
        })
    }
    /// Send a native mouse button press at physical child coordinates.
    pub fn press_button(
        &self,
        button: kivact_core::view::MouseButton,
        x: f64,
        y: f64,
    ) -> Result<()> {
        self.0.press_button(button, x, y).map_err(error)
    }
    /// Send a native mouse button release at physical child coordinates.
    pub fn release_button(
        &self,
        button: kivact_core::view::MouseButton,
        x: f64,
        y: f64,
    ) -> Result<()> {
        self.0.release_button(button, x, y).map_err(error)
    }
    /// Count native close requests received by the Windows host.
    // xtask-allow(os-branch): Native test environment availability is a test platform boundary.
    #[cfg(windows)]
    pub fn close_requests(&self) -> u32 {
        self.0.close_requests()
    }
    /// Send native file-drop messages to the Linux editor at physical coordinates.
    // xtask-allow(os-branch): Native test environment availability is a test platform boundary.
    #[cfg(target_os = "linux")]
    pub fn drop_files(&self, files: &[std::path::PathBuf], x: f64, y: f64) -> Result<()> {
        self.0.drop_files(files, x, y).map_err(error)
    }
    /// Close the parent; a remaining embedded App stops safely.
    pub fn close(&self) {
        self.0.close();
    }
    /// Whether the native parent has closed.
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}
