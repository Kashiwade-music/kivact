//! Role: Main-thread event loop, lifetime-bound CPU surface and callback panic boundary.
//!
//! The window outlives its surface; every callback runs on the creator thread.
//! The CPU surface (softbuffer) is created by the first CPU presentation, so
//! a window the GPU presents to never has one.
//!
//! What the runner adds to winit's events:
//! - Life cycle: winit's `suspended`, `resumed` and `memory_warning` are
//!   [`ShellEvent::Suspended`], [`ShellEvent::Resumed`] and
//!   [`ShellEvent::MemoryWarning`]. On Android a suspension destroys the
//!   window's surface: the CPU surface is dropped and
//!   [`ShellEvent::SurfaceDestroyed`] follows; the resumption gives the
//!   next surface generation ([`ShellEvent::SurfaceCreated`]). Desktops
//!   resume once, at start (the window is created then), and keep their
//!   first surface.
//! - Buttons and wheels: winit reports them without a position, so when
//!   the message's own position (Windows' subclass) differs from the last
//!   move, a mouse `Move` to it is delivered first. Every mouse event
//!   carries the last position and the held buttons
//!   ([`arvane_shell::Mouse`]); every press and release is counted
//!   ([`arvane_shell::ClickCounter`]).
//! - Wheels: a notch scrolls the system's lines (characters with Shift)
//!   of 20 logical px each, Shift turning a vertical notch sideways; a
//!   "page" setting (more than 100 lines) scrolls the window's height.
//!   Pixel deltas pass as they are, in logical px.
//! - The environment ([`WinitWindow::environment`]): a theme change, a
//!   move to a monitor of another identity (`Moved` or a scale change; the
//!   refresh period is read again first), a reduced motion, mouse settings
//!   or locale change (`WM_SETTINGCHANGE`) and a locale changed while the
//!   window had no focus (read again at `Focused(true)`) each arrive as one
//!   [`ShellEvent::EnvironmentChanged`]. winit tells of no locale change.
//! - Contacts: winit's touches become [`ShellEvent::Pointer`]s
//!   (`contacts.rs`); losing the focus cancels every contact in progress
//!   and, with a mouse button held, the mouse's gesture (one `Cancel`).
//!
//! Sizes, minimum sizes and IME areas are `kivact-types` logical sizes and
//! rectangles. A panic in a callback is caught here and ends the loop with
//! a [`ShellError`].
//!
//! Limits:
//! - A precision touchpad scrolls by winit's wheel lines: no pixel deltas
//!   and no inertia (DirectManipulation).
//! - Logoff and shutdown (`WM_QUERYENDSESSION`, `WM_ENDSESSION`) are not
//!   received, so no `CloseRequested` arrives then.
//! - winit 0.30 reports no safe area on any platform (it stays zero), nor
//!   the area a software keyboard covers.

use crate::a11y::native::NativeAccessibility;
use crate::key_code::physical_code;
use crate::native_input::state::NativeInput;
use crate::services::Services;
use arvane_shell::ShellImeText;
use arvane_shell::{ShellControl, ShellError, ShellOptions, ShellWaker};
use arvane_shell::{ShellEvent, ShellEventLoop, ShellHandler, ShellWindow};
use kivact_types::{
    Cursor as ShellCursor, Key as ShellKey, Modifiers as ShellModifiers, MouseButton as ShellButton,
};
use kivact_types::{Point, PointerPhase, Rect, Size};
use std::num::NonZeroU32;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, Ime, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{CursorIcon, Window, WindowId};

/// The CPU presentation surface of a window.
type CpuSurface = softbuffer::Surface<Arc<Window>, Arc<Window>>;

/// Native window and CPU surface; deliberately !Send and !Sync through Rc.
pub struct WinitWindow {
    /// The window's subclass (Windows), held for its drop, which removes
    /// it before the window goes (fields drop in order).
    #[cfg(windows)]
    _subclass: Option<crate::native_input::Subclass>,
    /// What the subclass and the runner share.
    input: Rc<NativeInput>,
    /// Created by the first CPU presentation (`present_with`).
    surface: Option<CpuSurface>,
    /// `Arc`: a GPU surface keeps the window it presents to alive on any
    /// thread (`gpu_target`); `affinity` keeps this
    /// handle on the UI thread.
    pub(crate) window: Arc<Window>,
    started: Instant,
    /// The generation of the window's presentation surface, `None` while
    /// the platform destroyed it (`surface_generation`).
    generation: Option<u64>,
    /// Surfaces the platform gave the window so far.
    surfaces: u64,
    // Rc enforces affinity even if an upstream platform makes its surface Send.
    affinity: Rc<()>,
    accessibility: NativeAccessibility,
    services: Services,
    /// The monitor the window was last seen on.
    monitor: Option<winit::monitor::MonitorHandle>,
    /// The refresh period of that monitor (`frame_period_ns`).
    frame_period_ns: u64,
}
impl ShellWindow for WinitWindow {
    fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: Duration) -> bool {
        crate::wait::until(done, limit)
    }

    /// Write UTF-8 text to the native clipboard.
    fn copy_text(&mut self, text: &str) -> Result<(), ShellError> {
        self.services.copy_text(text)
    }
    /// Read native clipboard text.
    fn read_text(&mut self) -> Result<String, ShellError> {
        self.services.read_text()
    }
    /// Show a window-parented file picker; cancellation returns None.
    fn pick_file(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError> {
        Services::pick_file(&self.window, request)
    }
    /// Show a window-parented multiple-file dialog; cancellation returns None.
    fn pick_files(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<Vec<std::path::PathBuf>>, ShellError> {
        Services::pick_files(&self.window, request)
    }
    /// Show a window-parented save dialog; cancellation returns None.
    fn save_file(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError> {
        Services::save_file(&self.window, request)
    }
    /// Show a window-parented folder picker; cancellation returns None.
    /// The request's file name and filters do not apply to a folder.
    fn pick_folder(
        &mut self,
        request: &arvane_shell::ShellFileDialog,
    ) -> Result<Option<std::path::PathBuf>, ShellError> {
        Services::pick_folder(&self.window, request)
    }
    /// Open a file or folder with the OS default handler (Explorer on
    /// Windows, `xdg-open` on Linux). `Ok` once the handler was started; a
    /// missing path or a handler that cannot be started is an error naming
    /// the path, and on other platforms (macOS) the error is
    /// [`ShellError::unsupported`].
    fn open_path(&mut self, path: &std::path::Path) -> Result<(), ShellError> {
        crate::open::open_path(path)
    }
    /// Show a file or folder selected in the OS file manager (Explorer's
    /// `/select,` on Windows). `xdg-open` cannot select, so on Linux the
    /// containing folder is opened instead. Errors as for
    /// [`WinitWindow::open_path`].
    fn reveal_path(&mut self, path: &std::path::Path) -> Result<(), ShellError> {
        crate::open::reveal_path(path)
    }
    /// Drop the CPU presentation surface after a presentation failure; the
    /// next CPU presentation creates a new one.
    fn recover_surface(&mut self) -> Result<(), ShellError> {
        self.surface = None;
        Ok(())
    }
    /// Whether the window shows a frame its CPU presentation surface
    /// presented (it presented a CPU frame since nothing else presented to
    /// it).
    fn shows_cpu_frame(&self) -> bool {
        self.surface.is_some()
    }
    /// Release the CPU presentation surface (and its pixel buffer) of a
    /// window something else (a GPU surface) presents to now.
    fn release_cpu_surface(&mut self) {
        self.surface = None;
    }
    /// The window as a GPU surface's target. winit hands window handles
    /// out only on the thread running the window's event loop: make the
    /// surface here.
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError> {
        make(self.window.clone());
        Ok(())
    }
    /// A key naming the window among the process's windows (test windows'
    /// keys have the top bit set).
    fn key(&self) -> u64 {
        u64::from(self.window.id())
    }
    /// Validate a complete accessibility tree and replace the window's
    /// accepted one with it: assistive technology is handed every node. A
    /// rejected tree keeps the accepted one.
    fn replace_accessibility(
        &mut self,
        tree: &kivact_types::SemanticsTree,
    ) -> Result<(), arvane_shell::AccessibilityError> {
        self.accessibility.replace(tree)
    }

    /// Validate and apply the accessibility nodes that may have changed:
    /// assistive technology is handed only those that did (nothing when
    /// none did). Without an accepted tree, or with a new root, `update`
    /// must hold the complete tree. A rejected update keeps the accepted
    /// tree.
    fn update_accessibility(
        &mut self,
        update: &kivact_types::SemanticsUpdate,
    ) -> Result<(), arvane_shell::AccessibilityError> {
        self.accessibility.update(update)
    }

    /// Whether assistive technology reads the window's accessibility tree:
    /// it asked for it (the shell then wakes the application) and has not
    /// let go of it (AT-SPI deactivates when the screen reader stops).
    /// Otherwise nothing reads the tree, so the application need not build
    /// it; the next activation wakes it to hand over the whole tree.
    fn accessibility_active(&self) -> bool {
        self.accessibility.active()
    }
    /// Current logical content size.
    fn size(&self) -> Size {
        let s = self
            .window
            .inner_size()
            .to_logical::<f32>(self.window.scale_factor());
        Size {
            width: s.width,
            height: s.height,
        }
    }
    /// Current physical content size.
    fn physical_size(&self) -> (u32, u32) {
        let s = self.window.inner_size();
        (s.width, s.height)
    }
    /// Physical pixels per logical pixel.
    fn scale(&self) -> f32 {
        self.window.scale_factor() as f32
    }
    /// Monotonic clock from loop creation.
    fn now_ns(&self) -> u64 {
        self.started.elapsed().as_nanos().min(u64::MAX as u128) as u64
    }
    /// What this backend can do on this platform through winit 0.30 (the
    /// facade fills in the GPU): touch arrives as touch except on macOS
    /// (a trackpad is not a touch screen); pens are told from touch on
    /// Windows only (the window subclass; elsewhere winit reports them as
    /// touch); the software keyboard and memory warnings exist on Android
    /// and iOS; winit reports no safe area anywhere; the IME's
    /// reconversion is answered on Windows (the subclass); AccessKit
    /// reaches assistive technology on desktops.
    fn capabilities(&self) -> kivact_types::Capabilities {
        let mobile = cfg!(any(target_os = "android", target_os = "ios"));
        kivact_types::Capabilities {
            touch: !cfg!(target_os = "macos"),
            pen: cfg!(windows) && self.input.subclassed.get(),
            soft_keyboard: SOFT_KEYBOARD,
            safe_area: false,
            memory_warnings: mobile,
            ime_reconversion: cfg!(windows) && self.input.subclassed.get(),
            native_accessibility: !mobile,
            gpu: false,
        }
    }
    /// The generation of the window's presentation surface: the count of
    /// surfaces the platform gave the window (the first is 1), or `None`
    /// while it has none ([`ShellEvent::SurfaceDestroyed`] until
    /// [`ShellEvent::SurfaceCreated`]). Desktops keep the first.
    fn surface_generation(&self) -> Option<u64> {
        self.generation
    }
    /// Schedule OS-driven redraw.
    fn request_redraw(&self) {
        self.window.request_redraw();
    }
    /// Change the title.
    fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }
    /// Change minimum logical content size.
    fn set_min_size(&self, size: Option<Size>) {
        self.window
            .set_min_inner_size(size.map(|s| LogicalSize::new(s.width, s.height)));
    }
    /// Whether the system asks to reduce motion (Windows: animation effects
    /// are off). A change arrives as [`ShellEvent::EnvironmentChanged`].
    fn reduced_motion(&self) -> bool {
        self.input.reduced_motion.get()
    }
    /// The window's environment now: scale, refresh period, reduce-motion
    /// and theme preferences, safe area (zero: no desktop covers a window's
    /// content with system UI; winit 0.30 reports none elsewhere either),
    /// mouse settings and the system's locale. A change arrives as
    /// [`ShellEvent::EnvironmentChanged`] (the scale's as
    /// [`ShellEvent::Resized`]).
    fn environment(&self) -> kivact_types::Environment {
        let (double_click_ns, double_click_distance) = self.double_click();
        let mouse = self.input.mouse.get();
        kivact_types::Environment {
            scale: self.scale(),
            frame_period_ns: self.frame_period_ns,
            reduced_motion: self.reduced_motion(),
            dark: self.window.theme() == Some(winit::window::Theme::Dark),
            safe_area: kivact_types::Insets::default(),
            mouse: kivact_types::MouseSettings {
                double_click_ns,
                double_click_distance,
                wheel_lines: mouse.wheel_lines,
                wheel_chars: mouse.wheel_chars,
            },
            locale: self.input.locale.borrow().clone(),
        }
    }

    fn request_size(&mut self, size: Size) -> Result<(), ShellError> {
        let _ = self
            .window
            .request_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
        Ok(())
    }
    /// Enter or leave borderless fullscreen.
    fn set_fullscreen(&self, fullscreen: bool) -> Result<(), ShellError> {
        self.window
            .set_fullscreen(fullscreen.then_some(winit::window::Fullscreen::Borderless(None)));
        Ok(())
    }
    /// Current fullscreen state.
    fn fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }
    /// Show `cursor` while the pointer is over the window's content.
    fn set_cursor(&self, cursor: ShellCursor) {
        self.window.set_cursor_visible(cursor != ShellCursor::None);
        self.window.set_cursor(match cursor {
            ShellCursor::Auto | ShellCursor::Default => CursorIcon::Default,
            ShellCursor::Text => CursorIcon::Text,
            ShellCursor::Pointer => CursorIcon::Pointer,
            ShellCursor::ColResize => CursorIcon::ColResize,
            ShellCursor::RowResize => CursorIcon::RowResize,
            ShellCursor::EwResize => CursorIcon::EwResize,
            ShellCursor::NsResize => CursorIcon::NsResize,
            ShellCursor::Grab => CursorIcon::Grab,
            ShellCursor::Grabbing => CursorIcon::Grabbing,
            ShellCursor::Crosshair => CursorIcon::Crosshair,
            ShellCursor::Move => CursorIcon::Move,
            ShellCursor::NotAllowed => CursorIcon::NotAllowed,
            ShellCursor::None => CursorIcon::Default,
        });
    }
    /// Enable IME when an editable control owns focus.
    fn set_ime_allowed(&self, allowed: bool) {
        self.window.set_ime_allowed(allowed);
    }
    /// Show or hide the software keyboard. winit 0.30 shows it with the IME
    /// on Android and iOS (`set_ime_allowed`); desktops have none, so the
    /// request does nothing there.
    fn set_soft_keyboard(&self, shown: bool) {
        if SOFT_KEYBOARD {
            self.window.set_ime_allowed(shown);
        }
    }
    /// Position the native IME candidate UI at a logical caret rectangle.
    fn set_ime_cursor_area(&self, area: Rect) {
        self.window.set_ime_cursor_area(
            LogicalPosition::new(area.x, area.y),
            LogicalSize::new(area.width, area.height),
        );
    }
    /// What the focused text input shows the IME (`None`: no text input
    /// has focus), answering the platform IME's queries about it
    /// (reconversion, the text around the composition, character
    /// positions; Windows).
    fn set_ime_text(&self, text: Option<ShellImeText>) {
        self.window
            .set_ime_purpose(if text.as_ref().is_some_and(|t| t.password) {
                winit::window::ImePurpose::Password
            } else {
                winit::window::ImePurpose::Normal
            });
        *self.input.text.borrow_mut() = text;
    }
    /// The refresh period (ns) of the display the window is on: its
    /// monitor's reported rate, a whole-hertz report read as the common
    /// rate it was rounded down from (Windows reports a 60 Hz mode as
    /// 59 Hz). An unknown or implausible report keeps the last period
    /// ([`arvane_shell::DEFAULT_FRAME_PERIOD_NS`] before any). A move to another
    /// monitor is announced as [`ShellEvent::EnvironmentChanged`].
    fn frame_period_ns(&self) -> u64 {
        self.frame_period_ns
    }

    /// Offer the current CPU buffer once and commit only on callback success.
    /// The caller retains its rendering error; backend failures stay separate.
    /// Missing surfaces and zero-sized frames do not invoke the callback.
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        draw: &mut dyn FnMut(&mut [u32]) -> bool,
    ) -> Result<bool, ShellError> {
        let _ = &self.affinity;
        if self.generation.is_none() || width == 0 || height == 0 {
            return Ok(false);
        }
        if (width, height) != self.physical_size() {
            return Err(ShellError::new("frame size differs from window surface"));
        }
        let width = NonZeroU32::new(width).expect("invariant: zero-sized frames returned above");
        let height = NonZeroU32::new(height).expect("invariant: zero-sized frames returned above");
        let surface = match &mut self.surface {
            Some(surface) => surface,
            None => self.surface.insert(cpu_surface(&self.window)?),
        };
        surface
            .resize(width, height)
            .map_err(|e| ShellError::new(e.to_string()))?;
        let mut buffer = surface
            .buffer_mut()
            .map_err(|e| ShellError::new(e.to_string()))?;
        if !draw(&mut buffer[..]) {
            return Ok(false);
        }
        buffer
            .present()
            .map(|()| true)
            .map_err(|e| ShellError::new(e.to_string()))
    }
}
/// A physical client position in logical pixels.
fn logical((x, y): (f64, f64), scale: f64) -> Point {
    Point {
        x: (x / scale) as f32,
        y: (y / scale) as f32,
    }
}

fn cpu_surface(window: &Arc<Window>) -> Result<CpuSurface, ShellError> {
    let context =
        softbuffer::Context::new(window.clone()).map_err(|e| ShellError::new(e.to_string()))?;
    softbuffer::Surface::new(&context, window.clone()).map_err(|e| ShellError::new(e.to_string()))
}
/// Whether the platform destroys a window's surface when it suspends the
/// application (Android), so the window has none until it resumes.
const SUSPENSION_DESTROYS_SURFACES: bool = cfg!(target_os = "android");
/// Whether the platform has a software keyboard winit can show.
pub(crate) const SOFT_KEYBOARD: bool = cfg!(any(target_os = "android", target_os = "ios"));
/// Main-thread window loop. Construct and run on the application's main thread.
pub struct Shell {
    event_loop: EventLoop<()>,
    options: ShellOptions,
}
impl Shell {
    /// Create the native event loop, rejecting invalid geometry and OS failures.
    pub fn new(options: ShellOptions) -> Result<Self, ShellError> {
        if !options.width.is_finite()
            || !options.height.is_finite()
            || options.width <= 0.0
            || options.height <= 0.0
        {
            return Err(ShellError::new(
                "initial window size must be positive and finite",
            ));
        }
        let event_loop = EventLoop::with_user_event()
            .build()
            .map_err(|e| ShellError::new(e.to_string()))?;
        Ok(Self {
            event_loop,
            options,
        })
    }
}
struct Runner<'a> {
    handler: &'a mut dyn ShellHandler,
    options: ShellOptions,
    window: Option<WinitWindow>,
    started: Instant,
    waker: ShellWaker,
    failure: Option<ShellError>,
    control: ShellControl,
    /// The last pointer position reported (physical client pixels).
    cursor: Option<(f64, f64)>,
    /// The mouse's logical position and held buttons, which winit's button
    /// and leave messages do not carry.
    mouse: arvane_shell::Mouse,
    /// Shift is held (a vertical wheel then scrolls horizontally).
    shift: bool,
    file_paths: Vec<std::path::PathBuf>,
    dropped_files: Vec<std::path::PathBuf>,
    /// Counts the presses of the pointer events delivered.
    clicks: arvane_shell::ClickCounter,
    /// The touch and pen contacts in progress.
    contacts: arvane_shell::Contacts,
}
impl Runner<'_> {
    fn file_position(&self) -> kivact_types::Point {
        self.window
            .as_ref()
            .and_then(|w| crate::native_input::file_position(&w.window))
            .unwrap_or_else(|| {
                let (x, y) = self.cursor.unwrap_or_default();
                let scale = self
                    .window
                    .as_ref()
                    .map_or(1.0, |w| w.window.scale_factor());
                kivact_types::Point {
                    x: (x / scale) as f32,
                    y: (y / scale) as f32,
                }
            })
    }
    /// Deliver a window's input event, its presses counted.
    fn input(&mut self, el: &ActiveEventLoop, mut event: ShellEvent) {
        if let Some(window) = &self.window {
            self.clicks
                .count(&mut event, || window.now_ns(), || window.double_click());
        }
        self.callback(el, Some(event));
    }
    fn callback(&mut self, el: &ActiveEventLoop, event: Option<ShellEvent>) {
        if self.failure.is_some() {
            return;
        }
        let Some(window) = self.window.as_mut() else {
            return;
        };
        let result = catch_unwind(AssertUnwindSafe(|| match event {
            Some(event) => self.handler.event(event, window),
            None => self.handler.start(window, self.waker.clone()),
        }));
        match result {
            Ok(Ok(control)) => {
                self.control = control;
                if control.exit {
                    el.exit();
                } else {
                    match control.next_wake_ns {
                        None => el.set_control_flow(ControlFlow::Wait),
                        Some(ns) => match self.started.checked_add(Duration::from_nanos(ns)) {
                            Some(deadline) => el.set_control_flow(ControlFlow::WaitUntil(deadline)),
                            None => {
                                self.failure = Some(ShellError::new(
                                    "shell wake deadline exceeds the platform clock range",
                                ));
                                el.exit();
                            }
                        },
                    }
                }
            }
            Ok(Err(error)) => {
                self.failure = Some(error);
                el.exit();
            }
            Err(payload) => {
                let message = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("non-string panic");
                self.failure = Some(ShellError::new(format!(
                    "shell callback panicked: {message}"
                )));
                el.exit();
            }
        }
    }
}
impl ApplicationHandler<()> for Runner<'_> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if let Some(window) = self.window.as_mut() {
            // A platform that destroyed the surface with the suspension
            // gives the window a new one now.
            if window.generation.is_none() {
                window.surfaces += 1;
                window.generation = Some(window.surfaces);
                self.callback(el, Some(ShellEvent::SurfaceCreated));
            }
            self.callback(el, Some(ShellEvent::Resumed));
            return;
        }
        let result = (|| {
            let logical = (
                f64::from(self.options.width),
                f64::from(self.options.height),
            );
            let mut attributes = Window::default_attributes()
                .with_visible(false)
                .with_title(&self.options.title)
                .with_inner_size(LogicalSize::new(self.options.width, self.options.height));
            let placement = crate::display::initial_window_position(el, logical);
            if let Some((position, _)) = &placement {
                attributes = attributes.with_position(*position);
            }
            let window = Arc::new(
                el.create_window(attributes)
                    .map_err(|e| ShellError::new(e.to_string()))?,
            );
            if let Some((_, placement)) = &placement {
                crate::display::center_window(placement, &window, logical);
            }
            let accessibility = NativeAccessibility::new(el, &window, self.waker.clone());
            let input = Rc::new(NativeInput::default());
            input.scale.set(window.scale_factor());
            input
                .reduced_motion
                .set(crate::native_input::reduced_motion());
            input
                .mouse
                .set(crate::native_input::state::MouseSettings::read());
            input
                .locale
                .replace(crate::native_input::locale::system_locale());
            *input.waker.borrow_mut() = Some(self.waker.clone());
            let monitor = window.current_monitor();
            let mut window = WinitWindow {
                #[cfg(windows)]
                _subclass: crate::native_input::Subclass::install(&window, &input),
                input,
                surface: None,
                window,
                started: self.started,
                generation: Some(1),
                surfaces: 1,
                affinity: Rc::new(()),
                accessibility,
                services: Services::default(),
                monitor,
                frame_period_ns: arvane_shell::DEFAULT_FRAME_PERIOD_NS,
            };
            window.read_frame_period();
            Ok::<_, ShellError>(window)
        })();
        match result {
            Ok(window) => {
                self.window = Some(window);
                self.callback(el, None);
                if self.failure.is_none()
                    && let Some(window) = &self.window
                {
                    window.window.set_visible(true);
                }
            }
            Err(e) => {
                self.failure = Some(e);
                el.exit();
            }
        }
    }
    fn suspended(&mut self, el: &ActiveEventLoop) {
        self.callback(el, Some(ShellEvent::Suspended));
        // Android destroys the native window's surface with the suspension
        // (winit asks applications to drop theirs); iOS keeps it.
        if SUSPENSION_DESTROYS_SURFACES && let Some(window) = self.window.as_mut() {
            window.generation = None;
            window.release_cpu_surface();
            self.callback(el, Some(ShellEvent::SurfaceDestroyed));
        }
    }
    fn memory_warning(&mut self, el: &ActiveEventLoop) {
        self.callback(el, Some(ShellEvent::MemoryWarning));
    }
    fn user_event(&mut self, el: &ActiveEventLoop, _: ()) {
        if !self.file_paths.is_empty() {
            let point = self
                .window
                .as_ref()
                .and_then(|w| w.input.file_pointer.take());
            if let Some(position) = point {
                self.input(
                    el,
                    ShellEvent::File {
                        phase: arvane_shell::FilePhase::Over,
                        files: self.file_paths.clone(),
                        position,
                    },
                );
            }
        }
        let environment = self
            .window
            .as_ref()
            .is_some_and(|window| window.input.env_changed.take());
        if environment {
            self.callback(el, Some(ShellEvent::EnvironmentChanged));
        }
        let actions = self
            .window
            .as_mut()
            .map(|window| window.accessibility.drain_actions())
            .unwrap_or_default();
        for action in actions {
            self.callback(el, Some(action));
        }
        self.callback(el, Some(ShellEvent::Wake));
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if !self.dropped_files.is_empty() {
            let files = std::mem::take(&mut self.dropped_files);
            let position = self.file_position();
            self.file_paths.clear();
            self.input(
                el,
                ShellEvent::File {
                    phase: arvane_shell::FilePhase::Drop,
                    files,
                    position,
                },
            );
        }
    }
    fn new_events(&mut self, el: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::ResumeTimeReached { .. }) {
            self.callback(el, Some(ShellEvent::Wake));
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_mut() else {
            return;
        };
        if window.window.id() != id {
            return;
        }
        window.accessibility.process_event(&window.window, &event);
        let scale = window.window.scale_factor();
        // The IME's character positions follow a scale change at once, even
        // when the text it answers from stays the same.
        window.input.scale.set(scale);
        let (mouse, height) = (window.input.mouse.get(), window.size().height);
        // What the window's subclass learned that winit does not report,
        // delivered before the event it precedes.
        let mut before = None;
        match &event {
            // A range the IME took for reconversion.
            WindowEvent::Ime(_) => {
                before = window
                    .input
                    .reconvert
                    .take()
                    .map(|(start, end)| ShellEvent::ImeSelection { start, end });
            }
            // winit reports buttons and wheels without a position: the
            // message's own, when it differs from the last move (a move
            // the platform reported late, after the button).
            WindowEvent::MouseInput { .. } | WindowEvent::MouseWheel { .. } => {
                let size = window.window.inner_size();
                let inside = |(x, y): (f64, f64)| {
                    x >= 0.0 && y >= 0.0 && x < f64::from(size.width) && y < f64::from(size.height)
                };
                if let Some(at) = window.input.pointer.take().filter(|at| inside(*at))
                    && self.cursor != Some(at)
                {
                    self.cursor = Some(at);
                    let at = logical(at, scale);
                    before = Some(ShellEvent::Pointer(self.mouse.event(
                        PointerPhase::Move,
                        Some(at),
                        None,
                    )));
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Some((position.x, position.y));
            }
            WindowEvent::CursorLeft { .. } => self.cursor = None,
            // winit tells of no change of the locale, and only Windows has a
            // message for it (`WM_SETTINGCHANGE`): coming back to the window
            // is when the user may have changed it elsewhere.
            WindowEvent::Focused(true) => {
                let locale = crate::native_input::locale::system_locale();
                if window.input.locale.replace(locale.clone()) != locale {
                    before = Some(ShellEvent::EnvironmentChanged);
                }
            }
            // Another monitor may refresh at another rate.
            WindowEvent::Moved(_) | WindowEvent::ScaleFactorChanged { .. } => {
                let monitor = window.window.current_monitor();
                if monitor != window.monitor {
                    window.monitor = monitor;
                    window.read_frame_period();
                    before = Some(ShellEvent::EnvironmentChanged);
                }
            }
            _ => {}
        }
        // Contacts begin, move and end several at a time: a Down of a
        // contact whose end was lost cancels it first, and losing the focus
        // (and the pointer capture) cancels every contact and, when a mouse
        // button is held, the mouse's gesture.
        let contacts = match &event {
            WindowEvent::Touch(touch) => {
                let notes = window.input.contacts.borrow();
                self.contacts
                    .deliver(crate::contacts::map(*touch, scale, &notes))
            }
            WindowEvent::Focused(false) => {
                let mut events = self.contacts.cancel();
                if let Some(cancel) = self.mouse.cancel() {
                    events.push(ShellEvent::Pointer(cancel));
                }
                events
            }
            _ => Vec::new(),
        };
        if let Some(before) = before {
            self.input(el, before);
        }
        for contact in contacts {
            self.input(el, contact);
        }
        let mapped = match event {
            WindowEvent::RedrawRequested => Some(ShellEvent::Redraw),
            WindowEvent::CloseRequested => Some(ShellEvent::CloseRequested),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                Some(ShellEvent::Resized)
            }
            WindowEvent::Focused(focus) => Some(ShellEvent::Focused(focus)),
            WindowEvent::CursorMoved { position, .. } if !self.file_paths.is_empty() => {
                Some(ShellEvent::File {
                    phase: arvane_shell::FilePhase::Over,
                    files: self.file_paths.clone(),
                    position: kivact_types::Point {
                        x: (position.x / scale) as f32,
                        y: (position.y / scale) as f32,
                    },
                })
            }
            WindowEvent::CursorMoved { position, .. } => {
                let at = logical((position.x, position.y), scale);
                Some(ShellEvent::Pointer(self.mouse.event(
                    PointerPhase::Move,
                    Some(at),
                    None,
                )))
            }
            // The mouse keeps its position and buttons: a button held keeps
            // its capture and is released outside.
            WindowEvent::CursorLeft { .. } => Some(ShellEvent::Pointer(self.mouse.event(
                PointerPhase::Leave,
                None,
                None,
            ))),
            WindowEvent::MouseInput { state, button, .. } => match button {
                winit::event::MouseButton::Left => Some(ShellButton::Primary),
                winit::event::MouseButton::Right => Some(ShellButton::Secondary),
                winit::event::MouseButton::Middle => Some(ShellButton::Middle),
                winit::event::MouseButton::Back => Some(ShellButton::Back),
                winit::event::MouseButton::Forward => Some(ShellButton::Forward),
                _ => None,
            }
            .map(|button| {
                ShellEvent::Pointer(self.mouse.event(
                    if state == ElementState::Pressed {
                        PointerPhase::Down
                    } else {
                        PointerPhase::Up
                    },
                    None,
                    Some(button),
                ))
            }),
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => crate::wheel::lines(
                        x,
                        y,
                        self.shift,
                        height,
                        mouse.wheel_lines,
                        mouse.wheel_chars,
                    ),
                    MouseScrollDelta::PixelDelta(p) => {
                        ((-p.x / scale) as f32, (-p.y / scale) as f32)
                    }
                };
                Some(ShellEvent::Wheel { x, y })
            }
            WindowEvent::ModifiersChanged(m) => {
                let m = m.state();
                self.shift = m.shift_key();
                Some(ShellEvent::Modifiers(ShellModifiers {
                    shift: m.shift_key(),
                    ctrl: m.control_key(),
                    alt: m.alt_key(),
                    meta: m.super_key(),
                }))
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Released => {
                Some(ShellEvent::KeyUp {
                    key: map_key(&event.logical_key),
                    code: physical_code(event.physical_key),
                })
            }
            WindowEvent::KeyboardInput { event, .. } => Some(ShellEvent::Key {
                key: map_key(&event.logical_key),
                physical: physical_letter(event.physical_key),
                code: physical_code(event.physical_key),
                repeat: event.repeat,
                text: event.text.map(|s| s.to_string()),
            }),
            WindowEvent::HoveredFile(path) => {
                let first = self.file_paths.is_empty();
                if !self.file_paths.contains(&path) {
                    self.file_paths.push(path);
                }
                if let Some(window) = &self.window {
                    crate::native_input::track_files(&window.window, true);
                }
                Some(ShellEvent::File {
                    phase: if first {
                        arvane_shell::FilePhase::Enter
                    } else {
                        arvane_shell::FilePhase::Over
                    },
                    files: self.file_paths.clone(),
                    position: self.file_position(),
                })
            }
            WindowEvent::HoveredFileCancelled => {
                if let Some(window) = &self.window {
                    crate::native_input::track_files(&window.window, false);
                }
                Some(ShellEvent::File {
                    phase: arvane_shell::FilePhase::Leave,
                    files: std::mem::take(&mut self.file_paths),
                    position: self.file_position(),
                })
            }
            WindowEvent::DroppedFile(path) => {
                if let Some(window) = &self.window {
                    crate::native_input::track_files(&window.window, false);
                }
                self.dropped_files.push(path);
                None
            }
            WindowEvent::Ime(Ime::Preedit(text, selection)) => {
                Some(ShellEvent::Preedit { text, selection })
            }
            WindowEvent::Ime(Ime::Commit(text)) => Some(ShellEvent::Commit(text)),
            WindowEvent::Ime(Ime::Disabled) => Some(ShellEvent::ImeDisabled),
            WindowEvent::ThemeChanged(_) => Some(ShellEvent::EnvironmentChanged),
            _ => None,
        };
        if let Some(event) = mapped {
            self.input(el, event);
        }
    }
}
/// The US-layout letter of a physical letter key (`KeyA`..=`KeyZ`).
fn physical_letter(key: PhysicalKey) -> Option<char> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };
    const LETTERS: [KeyCode; 26] = [
        KeyCode::KeyA,
        KeyCode::KeyB,
        KeyCode::KeyC,
        KeyCode::KeyD,
        KeyCode::KeyE,
        KeyCode::KeyF,
        KeyCode::KeyG,
        KeyCode::KeyH,
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
        KeyCode::KeyM,
        KeyCode::KeyN,
        KeyCode::KeyO,
        KeyCode::KeyP,
        KeyCode::KeyQ,
        KeyCode::KeyR,
        KeyCode::KeyS,
        KeyCode::KeyT,
        KeyCode::KeyU,
        KeyCode::KeyV,
        KeyCode::KeyW,
        KeyCode::KeyX,
        KeyCode::KeyY,
        KeyCode::KeyZ,
    ];
    let i = LETTERS.iter().position(|c| *c == code)?;
    Some(char::from(b'a' + u8::try_from(i).ok()?))
}
fn map_key(key: &Key) -> ShellKey {
    match key {
        Key::Character(s) => s.chars().next().map_or(ShellKey::Unidentified, |c| {
            if c == ' ' {
                ShellKey::Space
            } else {
                ShellKey::Char(c)
            }
        }),
        Key::Named(n) => match n {
            NamedKey::Enter => ShellKey::Enter,
            NamedKey::Tab => ShellKey::Tab,
            NamedKey::Space => ShellKey::Space,
            NamedKey::Backspace => ShellKey::Backspace,
            NamedKey::Delete => ShellKey::Delete,
            NamedKey::Escape => ShellKey::Escape,
            NamedKey::ArrowUp => ShellKey::ArrowUp,
            NamedKey::ArrowDown => ShellKey::ArrowDown,
            NamedKey::ArrowLeft => ShellKey::ArrowLeft,
            NamedKey::ArrowRight => ShellKey::ArrowRight,
            NamedKey::Home => ShellKey::Home,
            NamedKey::End => ShellKey::End,
            NamedKey::PageUp => ShellKey::PageUp,
            NamedKey::PageDown => ShellKey::PageDown,
            NamedKey::ContextMenu => ShellKey::ContextMenu,
            NamedKey::F1 => ShellKey::F(1),
            NamedKey::F2 => ShellKey::F(2),
            NamedKey::F3 => ShellKey::F(3),
            NamedKey::F4 => ShellKey::F(4),
            NamedKey::F5 => ShellKey::F(5),
            NamedKey::F6 => ShellKey::F(6),
            NamedKey::F7 => ShellKey::F(7),
            NamedKey::F8 => ShellKey::F(8),
            NamedKey::F9 => ShellKey::F(9),
            NamedKey::F10 => ShellKey::F(10),
            NamedKey::F11 => ShellKey::F(11),
            NamedKey::F12 => ShellKey::F(12),
            _ => ShellKey::Unidentified,
        },
        _ => ShellKey::Unidentified,
    }
}

impl WinitWindow {
    /// How close in time (ns) and space (logical pixels per axis) a second
    /// press of a button counts as a double click: the system's setting
    /// (Windows' defaults elsewhere: 500 ms, 4 pixels), read at each press.
    fn double_click(&self) -> (u64, (f32, f32)) {
        let mouse = self.input.mouse.get();
        let scale = self.window.scale_factor();
        let logical = |px: u32| (f64::from(px) / scale) as f32;
        (
            u64::from(mouse.double_click_ms) * 1_000_000,
            (
                logical(mouse.double_click_px.0),
                logical(mouse.double_click_px.1),
            ),
        )
    }
    /// Read the display's refresh period again (created, or moved to
    /// another monitor).
    fn read_frame_period(&mut self) {
        let rate = crate::display::refresh_rate_millihertz(self.window.current_monitor());
        if let Some(millihertz) = rate {
            self.frame_period_ns = crate::display::period_ns(millihertz);
        }
    }
}

impl ShellEventLoop for Shell {
    fn run(self: Box<Self>, handler: &mut dyn ShellHandler) -> Result<(), ShellError> {
        let proxy = self.event_loop.create_proxy();
        let waker = ShellWaker::new(move || {
            let _ = proxy.send_event(());
        });
        let mut app = Runner {
            handler,
            options: self.options,
            window: None,
            started: Instant::now(),
            waker,
            failure: None,
            control: ShellControl::default(),
            cursor: None,
            mouse: arvane_shell::Mouse::default(),
            file_paths: Vec::new(),
            dropped_files: Vec::new(),
            shift: false,
            clicks: arvane_shell::ClickCounter::default(),
            contacts: arvane_shell::Contacts::default(),
        };
        let result = self.event_loop.run_app(&mut app);
        if let Some(window) = app.window.as_mut()
            && std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                app.handler.close(window);
            }))
            .is_err()
        {
            app.failure.get_or_insert_with(|| {
                ShellError::new("a callback panicked while closing the window")
            });
        }
        result.map_err(|e| ShellError::new(e.to_string()))?;
        app.failure.map_or(Ok(()), Err)
    }
}
