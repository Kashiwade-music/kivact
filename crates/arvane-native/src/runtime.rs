//! Role: Main-thread event loop and lifetime-bound CPU surface.
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
//! rectangles. A panic in a callback propagates out of `run`: winit carries it
//! across its window procedure (Win32, macOS) and resumes it after the loop,
//! and the X11 and Wayland dispatch frames are all Rust.
//!
//! Limits:
//! - A precision touchpad scrolls by winit's wheel lines: no pixel deltas
//!   and no inertia (DirectManipulation).
//! - Logoff and shutdown (`WM_QUERYENDSESSION`, `WM_ENDSESSION`) are not
//!   received, so no `CloseRequested` arrives then.
//! - winit 0.30 reports no safe area on any platform (it stays zero), nor
//!   the area a software keyboard covers.

use crate::a11y::native::NativeAccessibility;
use crate::cpu::CpuSurface;
use crate::key_code::{map_key, physical_code, physical_letter};
use crate::native_input::state::NativeInput;
use crate::services::Services;
use arvane_shell::ShellImeText;
use arvane_shell::{ShellControl, ShellError, ShellOptions, ShellWaker};
use arvane_shell::{ShellEvent, ShellEventLoop, ShellHandler, ShellWindow};
use kivact_types::{
    Cursor as ShellCursor, Modifiers as ShellModifiers, MouseButton as ShellButton,
};
use kivact_types::{Point, PointerPhase, Rect, Size};
use std::num::{NonZeroU32, NonZeroU64};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, Ime, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorIcon, Window, WindowId};

/// The softbuffer surface of a winit window.
type WinitSurface = softbuffer::Surface<Arc<Window>, Arc<Window>>;

/// The window's presentation surface as the platform gives it.
enum Surface {
    /// The platform destroyed it with a suspension (Android); `last` is its generation.
    Destroyed { last: NonZeroU64 },
    /// The platform surface of `generation` (the first is 1) and the CPU surface on it.
    Live {
        generation: NonZeroU64,
        cpu: CpuSurface<Arc<Window>, Arc<Window>>,
    },
}

/// Native window and CPU surface; deliberately !Send and !Sync through Rc.
pub struct WinitWindow {
    /// The window's subclass (Windows), held for its drop, which removes
    /// it before the window goes (fields drop in order).
    #[cfg(windows)]
    subclass: Option<crate::native_input::Subclass>,
    /// What the subclass and the runner share.
    input: Rc<NativeInput>,
    /// The CPU surface inside is created by the first CPU presentation (`present_with`).
    surface: Surface,
    /// `Arc`: a GPU surface keeps the window it presents to alive on any
    /// thread (`gpu_target`); `affinity` keeps this
    /// handle on the UI thread.
    pub(crate) window: Arc<Window>,
    started: Instant,
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
        self.release_cpu_surface();
        Ok(())
    }
    /// Whether the window shows a frame its CPU presentation surface
    /// presented (it presented a CPU frame since nothing else presented to
    /// it).
    fn shows_cpu_frame(&self) -> bool {
        matches!(&self.surface, Surface::Live { cpu, .. } if cpu.shows_frame())
    }
    /// Release the CPU presentation surface (and its pixel buffer) of a
    /// window something else (a GPU surface) presents to now.
    fn release_cpu_surface(&mut self) {
        if let Surface::Live { cpu, .. } = &mut self.surface {
            cpu.release();
        }
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
    /// Validate a complete accessibility tree and replace the window's
    /// accepted one with it: assistive technology is handed every node. A
    /// rejected tree keeps the accepted one.
    fn replace_accessibility(
        &mut self,
        tree: &kivact_types::protocol::SemanticsTree,
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
        update: &kivact_types::protocol::SemanticsUpdate,
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
            pen: self.subclassed(),
            soft_keyboard: SOFT_KEYBOARD,
            safe_area: false,
            memory_warnings: mobile,
            ime_reconversion: self.subclassed(),
            native_accessibility: !mobile,
            gpu: false,
        }
    }
    /// The generation of the window's presentation surface: the count of
    /// surfaces the platform gave the window (the first is 1), or `None`
    /// while it has none ([`ShellEvent::SurfaceDestroyed`] until
    /// [`ShellEvent::SurfaceCreated`]). Desktops keep the first.
    fn surface_generation(&self) -> Option<u64> {
        match self.surface {
            Surface::Live { generation, .. } => Some(generation.get()),
            Surface::Destroyed { .. } => None,
        }
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
    fn environment(&self) -> kivact_types::protocol::Environment {
        let (double_click_ns, double_click_distance) = self.double_click();
        let mouse = self.input.mouse.get();
        kivact_types::protocol::Environment {
            scale: self.scale(),
            frame_period_ns: self.frame_period_ns,
            reduced_motion: self.reduced_motion(),
            dark: self.window.theme() == Some(winit::window::Theme::Dark),
            safe_area: kivact_types::Insets::default(),
            mouse: kivact_types::protocol::MouseSettings {
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
        let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return Ok(false);
        };
        let physical = self.physical_size();
        let Surface::Live { cpu, .. } = &mut self.surface else {
            return Ok(false);
        };
        if (width, height) != physical {
            return Err(ShellError::new("frame size differs from window surface"));
        }
        cpu.present(w, h, || cpu_surface(&self.window), draw)
    }
}
/// A physical client position in logical pixels.
fn logical((x, y): (f64, f64), scale: f64) -> Point {
    Point {
        x: (x / scale) as f32,
        y: (y / scale) as f32,
    }
}

fn cpu_surface(window: &Arc<Window>) -> Result<WinitSurface, ShellError> {
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
/// Where the runner is in the window's life.
enum Run {
    /// `resumed` has not created the window yet.
    Starting,
    /// The handler receives the window's events.
    Running(WinitWindow),
    /// The handler asked to exit (`failure` None) or failed: it gets no more events; `close`
    /// follows the loop.
    Stopped {
        window: WinitWindow,
        failure: Option<ShellError>,
    },
    /// The window could not be created: no handler started.
    Unopened(ShellError),
}
impl Run {
    fn live(&mut self) -> Option<&mut WinitWindow> {
        match self {
            Self::Running(window) => Some(window),
            Self::Starting | Self::Stopped { .. } | Self::Unopened(_) => None,
        }
    }
    fn live_ref(&self) -> Option<&WinitWindow> {
        match self {
            Self::Running(window) => Some(window),
            Self::Starting | Self::Stopped { .. } | Self::Unopened(_) => None,
        }
    }
}
/// Files dragged from the operating system over the window.
enum Dnd {
    /// No file drag over the window.
    Idle,
    /// These files are dragged over the window (winit reports each hovered file).
    Hovering(Vec<std::path::PathBuf>),
    /// These files were dropped; the drop is delivered when winit's batch ends (`about_to_wait`),
    /// after every `DroppedFile` of it.
    Dropped(Vec<std::path::PathBuf>),
}
struct Runner<'a> {
    handler: &'a mut dyn ShellHandler,
    options: ShellOptions,
    run: Run,
    started: Instant,
    waker: ShellWaker,
    /// The last pointer position reported (physical client pixels).
    cursor: Option<(f64, f64)>,
    /// The mouse's logical position and held buttons, which winit's button
    /// and leave messages do not carry.
    mouse: arvane_shell::Mouse,
    /// Shift is held (a vertical wheel then scrolls horizontally).
    shift: bool,
    dnd: Dnd,
    /// Counts the presses of the pointer events delivered.
    clicks: arvane_shell::ClickCounter,
    /// The touch and pen contacts in progress.
    contacts: arvane_shell::Contacts,
}
impl Runner<'_> {
    /// Stop delivering events: the loop ends after this callback and `close` follows it.
    fn stop(&mut self, failure: Option<ShellError>) {
        self.run = match std::mem::replace(&mut self.run, Run::Starting) {
            Run::Running(window) => Run::Stopped { window, failure },
            other => other,
        };
    }
    /// Deliver the files dropped in this batch of winit events, after every `DroppedFile` of it.
    fn deliver_drop(&mut self, el: &ActiveEventLoop) {
        match std::mem::replace(&mut self.dnd, Dnd::Idle) {
            Dnd::Dropped(files) => {
                let position = self.file_position();
                self.input(
                    el,
                    ShellEvent::File {
                        phase: arvane_shell::FilePhase::Drop,
                        files,
                        position,
                    },
                );
            }
            other => self.dnd = other,
        }
    }
    fn file_position(&self) -> kivact_types::Point {
        self.run
            .live_ref()
            .and_then(|w| crate::native_input::file_position(&w.window))
            .unwrap_or_else(|| {
                let (x, y) = self.cursor.unwrap_or_default();
                let scale = self.run.live_ref().map_or(1.0, |w| w.window.scale_factor());
                kivact_types::Point {
                    x: (x / scale) as f32,
                    y: (y / scale) as f32,
                }
            })
    }
    /// Deliver a window's input event, its presses counted.
    fn input(&mut self, el: &ActiveEventLoop, mut event: ShellEvent) {
        if let Some(window) = self.run.live_ref() {
            self.clicks
                .count(&mut event, || window.now_ns(), || window.double_click());
        }
        self.callback(el, Some(event));
    }
    fn callback(&mut self, el: &ActiveEventLoop, event: Option<ShellEvent>) {
        let Run::Running(window) = &mut self.run else {
            return;
        };
        let result = match event {
            Some(event) => self.handler.event(event, window),
            None => self.handler.start(window, self.waker.clone()),
        };
        match result {
            Ok(ShellControl::Exit) => {
                self.stop(None);
                el.exit();
            }
            Ok(ShellControl::Wait) => el.set_control_flow(ControlFlow::Wait),
            Ok(ShellControl::WakeAt(ns)) => {
                match self.started.checked_add(Duration::from_nanos(ns)) {
                    Some(deadline) => el.set_control_flow(ControlFlow::WaitUntil(deadline)),
                    None => {
                        self.stop(Some(ShellError::new(
                            "shell wake deadline exceeds the platform clock range",
                        )));
                        el.exit();
                    }
                }
            }
            Err(error) => {
                self.stop(Some(error));
                el.exit();
            }
        }
    }
}
impl ApplicationHandler<()> for Runner<'_> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if let Some(window) = self.run.live() {
            // A platform that destroyed the surface with the suspension
            // gives the window a new one now.
            if let Surface::Destroyed { last } = window.surface {
                window.surface = Surface::Live {
                    generation: last.saturating_add(1),
                    cpu: CpuSurface::Released,
                };
                self.callback(el, Some(ShellEvent::SurfaceCreated));
            }
            self.callback(el, Some(ShellEvent::Resumed));
            return;
        }
        if !matches!(self.run, Run::Starting) {
            // A stopped runner resumes nothing.
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
                subclass: crate::native_input::Subclass::install(&window, &input),
                input,
                surface: Surface::Live {
                    generation: NonZeroU64::MIN,
                    cpu: CpuSurface::Released,
                },
                window,
                started: self.started,
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
                self.run = Run::Running(window);
                self.callback(el, None);
                if let Some(window) = self.run.live_ref() {
                    window.window.set_visible(true);
                }
            }
            Err(e) => {
                self.run = Run::Unopened(e);
                el.exit();
            }
        }
    }
    fn suspended(&mut self, el: &ActiveEventLoop) {
        self.callback(el, Some(ShellEvent::Suspended));
        // Android destroys the native window's surface with the suspension
        // (winit asks applications to drop theirs); iOS keeps it.
        if SUSPENSION_DESTROYS_SURFACES
            && let Some(window) = self.run.live()
            && let Surface::Live { generation, .. } = window.surface
        {
            // The CPU surface drops with the variant.
            window.surface = Surface::Destroyed { last: generation };
            self.callback(el, Some(ShellEvent::SurfaceDestroyed));
        }
    }
    fn memory_warning(&mut self, el: &ActiveEventLoop) {
        self.callback(el, Some(ShellEvent::MemoryWarning));
    }
    fn user_event(&mut self, el: &ActiveEventLoop, _: ()) {
        if let Dnd::Hovering(files) = &self.dnd
            && let Some(position) = self
                .run
                .live_ref()
                .and_then(|w| w.input.file_pointer.take())
        {
            let files = files.clone();
            self.input(
                el,
                ShellEvent::File {
                    phase: arvane_shell::FilePhase::Over,
                    files,
                    position,
                },
            );
        }
        let environment = self
            .run
            .live_ref()
            .is_some_and(|window| window.input.env_changed.take());
        if environment {
            self.callback(el, Some(ShellEvent::EnvironmentChanged));
        }
        let actions = self
            .run
            .live()
            .map(|window| window.accessibility.drain_actions())
            .unwrap_or_default();
        for action in actions {
            self.callback(el, Some(action));
        }
        self.callback(el, Some(ShellEvent::Wake));
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        self.deliver_drop(el);
    }
    fn new_events(&mut self, el: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::ResumeTimeReached { .. }) {
            self.callback(el, Some(ShellEvent::Wake));
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.run.live() else {
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
                    before = self
                        .mouse
                        .event(PointerPhase::Move, Some(at))
                        .map(ShellEvent::Pointer);
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
            WindowEvent::CursorMoved { position, .. } => {
                let at = logical((position.x, position.y), scale);
                match &self.dnd {
                    Dnd::Hovering(files) => Some(ShellEvent::File {
                        phase: arvane_shell::FilePhase::Over,
                        files: files.clone(),
                        position: at,
                    }),
                    Dnd::Idle | Dnd::Dropped(_) => self
                        .mouse
                        .event(PointerPhase::Move, Some(at))
                        .map(ShellEvent::Pointer),
                }
            }
            // The mouse keeps its position and buttons: a button held keeps
            // its capture and is released outside.
            WindowEvent::CursorLeft { .. } => self
                .mouse
                .event(PointerPhase::Leave, None)
                .map(ShellEvent::Pointer),
            WindowEvent::MouseInput { state, button, .. } => match button {
                winit::event::MouseButton::Left => Some(ShellButton::Primary),
                winit::event::MouseButton::Right => Some(ShellButton::Secondary),
                winit::event::MouseButton::Middle => Some(ShellButton::Middle),
                winit::event::MouseButton::Back => Some(ShellButton::Back),
                winit::event::MouseButton::Forward => Some(ShellButton::Forward),
                _ => None,
            }
            .and_then(|button| {
                self.mouse
                    .button(button, state == ElementState::Pressed, None)
                    .map(ShellEvent::Pointer)
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
                if matches!(self.dnd, Dnd::Dropped(_)) {
                    self.deliver_drop(el);
                }
                let (phase, files) = match &mut self.dnd {
                    Dnd::Hovering(files) => {
                        if !files.contains(&path) {
                            files.push(path);
                        }
                        (arvane_shell::FilePhase::Over, files.clone())
                    }
                    other => {
                        let files = vec![path];
                        *other = Dnd::Hovering(files.clone());
                        (arvane_shell::FilePhase::Enter, files)
                    }
                };
                if let Some(window) = self.run.live_ref() {
                    crate::native_input::track_files(&window.window, true);
                }
                Some(ShellEvent::File {
                    phase,
                    files,
                    position: self.file_position(),
                })
            }
            WindowEvent::HoveredFileCancelled => {
                match std::mem::replace(&mut self.dnd, Dnd::Idle) {
                    Dnd::Hovering(files) => {
                        if let Some(window) = self.run.live_ref() {
                            crate::native_input::track_files(&window.window, false);
                        }
                        Some(ShellEvent::File {
                            phase: arvane_shell::FilePhase::Leave,
                            files,
                            position: self.file_position(),
                        })
                    }
                    other => {
                        self.dnd = other;
                        None
                    }
                }
            }
            WindowEvent::DroppedFile(path) => {
                if let Some(window) = self.run.live_ref() {
                    crate::native_input::track_files(&window.window, false);
                }
                self.dnd = match std::mem::replace(&mut self.dnd, Dnd::Idle) {
                    Dnd::Dropped(mut files) => {
                        files.push(path);
                        Dnd::Dropped(files)
                    }
                    Dnd::Idle | Dnd::Hovering(_) => Dnd::Dropped(vec![path]),
                };
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
impl WinitWindow {
    /// Whether the Windows subclass is installed: it answers the IME's reconversion and tells pens
    /// from touch.
    fn subclassed(&self) -> bool {
        #[cfg(windows)]
        {
            self.subclass.is_some()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }
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
            run: Run::Starting,
            started: Instant::now(),
            waker,
            cursor: None,
            mouse: arvane_shell::Mouse::default(),
            dnd: Dnd::Idle,
            shift: false,
            clicks: arvane_shell::ClickCounter::default(),
            contacts: arvane_shell::Contacts::default(),
        };
        let result = self.event_loop.run_app(&mut app);
        let Runner { handler, run, .. } = app;
        let failure = match run {
            Run::Starting => None,
            Run::Running(mut window) => {
                handler.close(&mut window);
                None
            }
            Run::Stopped {
                mut window,
                failure,
            } => {
                handler.close(&mut window);
                failure
            }
            Run::Unopened(error) => Some(error),
        };
        result.map_err(|e| ShellError::new(e.to_string()))?;
        failure.map_or(Ok(()), Err)
    }
}
