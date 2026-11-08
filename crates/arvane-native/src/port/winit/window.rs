//! Role: A winit window with its CPU presentation surface and platform services.

use crate::a11y::native::NativeAccessibility;
use crate::native_input::state::NativeInput;
use crate::present::CpuSurface;
use crate::services::Services;
use arvane_shell::ShellImeText;
use arvane_shell::pipeline::ImePort;
use arvane_shell::{ShellError, ShellWindow};
use kivact_types::{Cursor as ShellCursor, PlatformRequest, PlatformResult, Rect, Size};
use std::num::{NonZeroU32, NonZeroU64};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::window::{CursorIcon, Window};

/// The softbuffer surface of a winit window.
type WinitSurface = softbuffer::Surface<Arc<Window>, Arc<Window>>;

/// The window's presentation surface as the platform gives it.
pub(super) enum Surface {
    /// The platform destroyed it with a suspension (Android); `last` is its generation.
    Destroyed { last: NonZeroU64 },
    /// The platform surface of `generation` (the first is 1) and the CPU surface on it.
    Live {
        generation: NonZeroU64,
        cpu: CpuSurface<Arc<Window>, Arc<Window>>,
    },
}

// xtask-allow(state-fields): Native resources and monitor cache; optional Windows subclass and last observed monitor are independent platform facts.
/// Native window and CPU surface; deliberately !Send and !Sync through Rc.
pub struct WinitWindow {
    // xtask-allow(state-fields): RAII guard + monitor cache (HOST-REWRITE module) - the Windows subclass held for its drop, which removes it before the window goes
    /// The window's subclass (Windows), held for its drop, which removes
    /// it before the window goes (fields drop in order).
    #[cfg(windows)]
    pub(super) subclass: Option<crate::native_input::Subclass>,
    /// What the subclass and the runner share.
    pub(super) input: Rc<NativeInput>,
    /// The CPU surface inside is created by the first CPU presentation (`present_with`).
    pub(super) surface: Surface,
    /// `Arc`: a GPU surface keeps the window it presents to alive on any
    /// thread (`gpu_target`); `affinity` keeps this
    /// handle on the UI thread.
    pub(crate) window: Arc<Window>,
    pub(super) started: Instant,
    // Rc enforces affinity even if an upstream platform makes its surface Send.
    pub(super) affinity: Rc<()>,
    pub(super) accessibility: NativeAccessibility,
    pub(super) services: Services,
    // xtask-allow(state-fields): RAII guard + monitor cache (HOST-REWRITE module) - the monitor the window was last seen on
    /// The monitor the window was last seen on.
    pub(super) monitor: Option<winit::monitor::MonitorHandle>,
    /// The refresh period of that monitor (`frame_period_ns`).
    pub(super) frame_period_ns: u64,
}
impl ShellWindow for WinitWindow {
    fn perform(&mut self, request: PlatformRequest) -> PlatformResult {
        match request {
            PlatformRequest::SaveState { .. } => return PlatformResult::Done,
            PlatformRequest::CopyText(text) => self
                .services
                .copy_text(&text)
                .map(|()| PlatformResult::Done),
            PlatformRequest::ReadText => self.services.read_text().map(PlatformResult::Text),
            PlatformRequest::PickFile(request) => {
                Services::pick_file(&self.window, &request).map(PlatformResult::Path)
            }
            PlatformRequest::PickFiles(request) => {
                Services::pick_files(&self.window, &request).map(PlatformResult::Paths)
            }
            PlatformRequest::SaveFile(request) => {
                Services::save_file(&self.window, &request).map(PlatformResult::Path)
            }
            PlatformRequest::PickFolder(request) => {
                Services::pick_folder(&self.window, &request).map(PlatformResult::Path)
            }
            PlatformRequest::OpenPath(path) => {
                crate::open::open_path(&path).map(|()| PlatformResult::Done)
            }
            PlatformRequest::RevealPath(path) => {
                crate::open::reveal_path(&path).map(|()| PlatformResult::Done)
            }
            PlatformRequest::FocusWindow => self.focus().map(|()| PlatformResult::Done),
            PlatformRequest::RequestWindowSize(size) => {
                let _ = self
                    .window
                    .request_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
                Ok(PlatformResult::Done)
            }
            PlatformRequest::Fullscreen(fullscreen) => {
                self.window.set_fullscreen(
                    fullscreen.then_some(winit::window::Fullscreen::Borderless(None)),
                );
                Ok(PlatformResult::Done)
            }
        }
        .unwrap_or_else(|error| match error {
            ShellError::Unsupported(_) => PlatformResult::Unsupported,
            ShellError::Failed(failure) => PlatformResult::Failed(failure),
        })
    }

    /// Bring the window to the front and give it the keyboard focus.
    fn focus(&mut self) -> Result<(), ShellError> {
        self.window.focus_window();
        Ok(())
    }

    fn cpu_format(&self) -> arvane_shell::CpuFormat {
        arvane_shell::CpuFormat::Xrgb8888
    }

    fn keeps_frames(&self) -> bool {
        false
    }
    fn keep_frame(&mut self, _: kivact_types::render::CpuFrame) {
        unreachable!("invariant: native windows do not keep frames")
    }

    fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: Duration) -> bool {
        crate::wait::until(done, limit)
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
        tree: &kivact_types::render::SemanticsTree,
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
        update: &kivact_types::render::SemanticsUpdate,
    ) -> Result<(), arvane_shell::AccessibilityError> {
        self.accessibility.update(update)
    }

    /// Whether assistive technology reads the window's accessibility tree:
    /// it asked for it (the shell then wakes the application) and has not
    /// let go of it (AT-SPI deactivates when the screen reader stops).
    /// Otherwise nothing reads the tree, so the application need not build
    /// it; the next activation wakes it to hand over the whole tree.
    /// Current logical content size.
    fn size(&self) -> Size {
        let s = self
            .window
            .inner_size()
            .to_logical::<f64>(self.window.scale_factor());
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
    fn scale(&self) -> f64 {
        self.window.scale_factor()
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
            state_restoration: false,
            clipboard_text: true,
            file_dialogs: true,
            open_path: cfg!(any(
                windows,
                all(
                    unix,
                    not(any(
                        target_os = "macos",
                        target_os = "ios",
                        target_os = "android"
                    ))
                )
            )),
            reveal_path: cfg!(any(
                windows,
                all(
                    unix,
                    not(any(
                        target_os = "macos",
                        target_os = "ios",
                        target_os = "android"
                    ))
                )
            )),
            fullscreen: true,
            min_size: true,
            command_key: if cfg!(target_os = "macos") {
                kivact_types::CommandKey::Command
            } else {
                kivact_types::CommandKey::Ctrl
            },
            context_menu_on: if cfg!(target_os = "macos") {
                kivact_types::ContextMenuTrigger::Press
            } else {
                kivact_types::ContextMenuTrigger::Release
            },
            touch: !cfg!(target_os = "macos"),
            pen: self.subclassed(),
            soft_keyboard: SOFT_KEYBOARD,
            safe_area: false,
            memory_warnings: mobile,
            ime_reconversion: self.subclassed(),
            native_accessibility: !mobile,
            gpu: false,
            multi_window: true,
            window_placement: false,
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
    fn environment(&self) -> kivact_types::Environment {
        let (double_click_ns, double_click_distance) = self.double_click();
        let mouse = self.input.mouse.get();
        kivact_types::Environment {
            scale_factor: self.scale(),
            frame_period_ns: self.frame_period_ns,
            reduced_motion: self.reduced_motion(),
            theme_hint: if self.window.theme() == Some(winit::window::Theme::Dark) {
                kivact_types::ThemeHint::Dark
            } else {
                kivact_types::ThemeHint::Light
            },
            safe_area: kivact_types::Insets::default(),
            pointer: kivact_types::PointerSettings {
                double_click_ns,
                double_click_distance,
                wheel_lines: mouse.wheel_lines,
                wheel_chars: mouse.wheel_chars,
                ..mouse
            },
            window_size: self.size(),
            fullscreen: self.fullscreen(),
            locale: self.input.locale.borrow().clone(),
            ..Default::default()
        }
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
    fn set_ime_purpose(&self, purpose: kivact_types::InputPurpose) {
        self.window
            .set_ime_purpose(if purpose == kivact_types::InputPurpose::Password {
                winit::window::ImePurpose::Password
            } else {
                winit::window::ImePurpose::Normal
            });
    }
    fn set_ime_allowed(&self, allowed: bool) {
        self.input
            .ime
            .borrow_mut()
            .set_allowed(&mut ImeWindow(self), allowed);
    }
    /// Show or hide the software keyboard. winit 0.30 shows it with the IME
    /// on Android and iOS (`set_ime_allowed`); desktops have none, so the
    /// request does nothing there.
    fn set_soft_keyboard(&self, shown: bool) {
        if SOFT_KEYBOARD {
            // Through the IME cache: the same native switch backs both.
            self.set_ime_allowed(shown);
        }
    }
    /// Position the native IME candidate UI at a logical caret rectangle.
    fn set_ime_cursor_area(&self, area: Rect) {
        // Native pixels keep a scale change visible to the shared output cache.
        let scale = self.window.scale_factor();
        let area = Rect {
            x: area.x * scale,
            y: area.y * scale,
            width: area.width * scale,
            height: area.height * scale,
        };
        self.input
            .ime
            .borrow_mut()
            .set_cursor_area(&mut ImeWindow(self), area);
    }
    /// What the focused text input shows the IME (`None`: no text input
    /// has focus), answering the platform IME's queries about it
    /// (reconversion, the text around the composition, character
    /// positions; Windows).
    fn set_ime_text(&self, text: Option<ShellImeText>) {
        self.input
            .ime
            .borrow_mut()
            .set_text(&mut ImeWindow(self), text);
    }
    /// The refresh period (ns) of the display the window is on: its
    /// monitor's reported rate, a whole-hertz report read as the common
    /// rate it was rounded down from (Windows reports a 60 Hz mode as
    /// 59 Hz). An unknown or implausible report keeps the last period
    /// ([`kivact_types::defaults::FRAME_PERIOD_NS`] before any). A move to another
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
        damage: Option<&[kivact_types::render::PixelRect]>,
        draw: &mut dyn FnMut(&mut [u32], arvane_shell::BufferAge) -> bool,
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
        cpu.present(w, h, || cpu_surface(&self.window), damage, draw)
    }
}

fn cpu_surface(window: &Arc<Window>) -> Result<WinitSurface, ShellError> {
    crate::present::surface(window)
}

/// Whether the platform has a software keyboard winit can show.
const SOFT_KEYBOARD: bool = cfg!(any(target_os = "android", target_os = "ios"));

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
    pub(super) fn double_click(&self) -> (u64, (f64, f64)) {
        let mouse = crate::native_input::pointer_settings(self.scale());
        (
            mouse.double_click_ns,
            (mouse.double_click_distance.0, mouse.double_click_distance.1),
        )
    }
    /// Read the display's refresh period again (created, or moved to
    /// another monitor).
    pub(super) fn read_frame_period(&mut self) {
        let rate = crate::display::refresh_rate_millihertz(self.window.current_monitor());
        if let Some(millihertz) = rate {
            self.frame_period_ns = crate::display::period_ns(millihertz);
        }
    }
}

struct ImeWindow<'a>(&'a WinitWindow);
impl ImePort for ImeWindow<'_> {
    fn allow(&mut self, allowed: bool) {
        self.0.window.set_ime_allowed(allowed);
    }
    fn cursor_area(&mut self, area: Rect) {
        self.0.window.set_ime_cursor_area(
            PhysicalPosition::new(area.x, area.y),
            PhysicalSize::new(area.width, area.height),
        );
    }
    fn surrounding(&mut self, text: Option<&ShellImeText>) {
        *self.0.input.text.borrow_mut() = text.cloned();
    }
}
