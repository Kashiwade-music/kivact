//! Role: VirtualWindow: an in-memory test window and the controls tests drive.

use super::shell::{TestSurface, VirtualShell, physical};
use crate::env::Env;
use crate::presentation::Notes;
use crate::presenter::Presenter;
use crate::{
    Accessibility, AccessibilityError, Error, ErrorKind, FramePlan, Presented, Renderer,
    RendererChange, ShellError, ShellFileDialog, ShellImeText, Window,
};
use arvane_native::test_display::{TestDisplay, TestWindow, TestWindowOptions};
use arvane_shell::ShellWindow;
use kivact_types::protocol::{Environment, Scene, SemanticsTree, SemanticsUpdate};
use kivact_types::{Capabilities, Cursor, GpuPreference, Insets, Rect, Size};
#[cfg(feature = "gpu-interop")]
use kivact_types::{ExternalImageId, GpuAdapter, SharedTextures};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Whether a [`VirtualWindow`] mirrors to a real window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowMode {
    /// A real window; failing to open one is an error.
    Real,
    /// No window: frames are rendered and kept, nothing is shown.
    Headless,
    /// A real window when one can be opened, else headless. The
    /// environment variable `ARVANE_TEST_HEADLESS=1` makes it headless.
    Auto,
}

/// Options of [`VirtualWindow::open`].
///
/// The default is an untitled 800 by 600 window at scale 1 on a 60 Hz
/// display, with the system's preferences off, mirrored to a real window
/// when one can be opened ([`WindowMode::Auto`]) and drawn on the CPU.
#[derive(Clone, Debug)]
pub struct VirtualWindowOptions {
    /// Title of the real window.
    pub title: String,
    /// Logical content size.
    pub size: Size,
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// The display's refresh period (ns).
    pub frame_period_ns: u64,
    /// The system's reduce-motion preference.
    pub reduced_motion: bool,
    /// The system prefers a dark theme.
    pub dark: bool,
    /// The system's locale as a BCP 47 tag (`None`: it cannot tell).
    pub locale: Option<String>,
    /// Window mirroring.
    pub mode: WindowMode,
    /// Draw frames on the GPU: through a GPU surface of the real window
    /// (read back), or offscreen when headless. Opening fails when the GPU
    /// renderer cannot start ([`gpu_unavailable`]).
    pub gpu: bool,
    /// The renderer forced over `gpu`, as `ARVANE_RENDERER` forces a
    /// native window's (`Some(Renderer::Cpu)` stands for
    /// `ARVANE_RENDERER=cpu`). A test window never reads the variable, so
    /// its frames are the same whatever the process's environment.
    pub renderer: Option<Renderer>,
    /// How the window's GPU device is made, as a native window's
    /// ([`crate::Shell::gpu_preference`]; `WGPU_BACKEND` and
    /// `WGPU_POWER_PREF` override it).
    pub gpu_preference: GpuPreference,
}

impl Default for VirtualWindowOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            size: Size {
                width: 800.0,
                height: 600.0,
            },
            scale: 1.0,
            frame_period_ns: 16_666_667,
            reduced_motion: false,
            dark: false,
            locale: None,
            mode: WindowMode::Auto,
            gpu: false,
            renderer: None,
            gpu_preference: GpuPreference::default(),
        }
    }
}

/// A test's window: what a native window does, in memory (see the module
/// documentation).
pub struct VirtualWindow {
    /// The headless shell: the window's state and services.
    shell: VirtualShell,
    /// The rasterizer of the last frame.
    renderer: Option<Renderer>,
    presenter: Presenter,
    /// The presenter's notes not taken yet.
    changes: Arc<Mutex<Vec<RendererChange>>>,
}

impl Drop for VirtualWindow {
    fn drop(&mut self) {
        self.presenter.shutdown(&mut self.shell);
    }
}

impl VirtualWindow {
    /// Open the window `options` describe, with a real window when the
    /// mode asks for one (failing to open one is an error only in
    /// [`WindowMode::Real`]).
    pub fn open(options: VirtualWindowOptions) -> Result<Self, Error> {
        let env = Env::read(options.gpu_preference);
        let changes = Arc::new(Mutex::new(Vec::new()));
        let noted = Arc::clone(&changes);
        let mut presenter = Presenter::new(
            Notes::new(Some(Box::new(move |change: &RendererChange| {
                noted
                    .lock()
                    .expect("renderer change log lock poisoned: a panic already happened")
                    .push(change.clone());
            }))),
            &env,
        );
        let gpu = crate::env::starts_gpu(options.renderer, options.gpu);
        if gpu {
            presenter.enable_gpu()?;
        }
        let real = open_real(&options, &env, physical(options.size, options.scale))?;
        Ok(Self {
            shell: VirtualShell {
                now: 0,
                size: options.size,
                requested_sizes: Vec::new(),
                scale: options.scale,
                period: options.frame_period_ns.max(1),
                reduced_motion: options.reduced_motion,
                dark: options.dark,
                locale: options.locale.clone(),
                safe_area: Insets::default(),
                clipboard: String::new(),
                dialogs: VecDeque::new(),
                opened: Vec::new(),
                revealed: Vec::new(),
                fail_next: None,
                redraw: Cell::new(false),
                title: RefCell::new(String::new()),
                min_size: Cell::new(None),
                ime: Cell::new(None),
                ime_text: RefCell::new(None),
                cursor: Cell::new(Cursor::Default),
                fullscreen: Cell::new(false),
                soft_keyboard: (Cell::new(false), Cell::new(0)),
                frame: None,
                gpu,
                real,
                headless: Accessibility::new(),
                clicks: arvane_shell::ClickCounter::default(),
                mouse: arvane_shell::Mouse::default(),
                contacts: arvane_shell::Contacts::default(),
                occluded: false,
                failing: 0,
                surface: TestSurface::Live {
                    generation: NonZeroU64::MIN,
                },
                cpu_frame: Cell::new(false),
            },
            renderer: None,
            presenter,
            changes,
        })
    }

    /// Requested logical content sizes, in order.
    pub fn requested_sizes(&self) -> &[Size] {
        &self.shell.requested_sizes
    }

    /// An event injected into the window as the platform would deliver it:
    /// its presses are counted ([`crate::ShellEvent::Pointer`]'s
    /// `clicks`) at the virtual time with Windows' default double-click
    /// time and distance (500 ms, 4 logical pixels). A mouse event becomes
    /// the window's mouse state ([`Self::mouse`] continues from it).
    pub fn input(&mut self, mut event: crate::ShellEvent) -> crate::ShellEvent {
        if let crate::ShellEvent::Pointer(input) = &event {
            self.shell.mouse.observe(*input);
        }
        // A focus loss injected as it is (not through `focus`) releases the
        // mouse's buttons all the same.
        if matches!(&event, crate::ShellEvent::Focused(false)) {
            let _ = self.shell.mouse.cancel();
        }
        let now = self.shell.now;
        self.shell
            .clicks
            .count(&mut event, || now, || arvane_shell::DEFAULT_DOUBLE_CLICK);
        event
    }

    /// The mouse event of `phase` as a native window makes it: at
    /// `position` (`None`: where the mouse last was), with the buttons held
    /// before it plus a `Down`'s `button` or less an `Up`'s. Deliver it
    /// through [`Self::input`], which counts its clicks. `None` when `position` is `None` and the mouse
    /// was never placed: a native window reports nothing then.
    pub fn mouse(
        &mut self,
        phase: kivact_types::PointerPhase,
        position: Option<kivact_types::Point>,
        button: Option<kivact_types::MouseButton>,
    ) -> Option<crate::ShellEvent> {
        self.shell
            .mouse
            .event(phase, position, button)
            .map(crate::ShellEvent::Pointer)
    }

    /// A touch or pen contact as the platform reports it, delivered as a
    /// native window delivers its contacts: `contact.id` is the
    /// platform's id of the contact (it may be reused once the contact
    /// ended) and `contact.kind` counts at its `Down`. The event carries a
    /// pointer id of the contact's own (a new one per contact, from 2; 1
    /// is the mouse); none when the window reports nothing (a move or an
    /// end of a contact that never began), and a `Cancel` of the old
    /// contact first for a `Down` of a platform id still in contact. Taps
    /// are counted (`clicks`) as [`Self::input`] counts presses.
    pub fn contact(
        &mut self,
        contact: kivact_types::protocol::PointerInput,
    ) -> Vec<crate::ShellEvent> {
        let mut events = self.shell.contacts.deliver(contact);
        for event in &mut events {
            *event = self.input(event.clone());
        }
        events
    }

    /// The platform gives the window the focus or takes it away: the
    /// events the window delivers, as a native window delivers them (losing
    /// the focus cancels every contact in progress and, with a mouse button
    /// held, the mouse's gesture first).
    pub fn focus(&mut self, focused: bool) -> Vec<crate::ShellEvent> {
        let mut events = if focused {
            Vec::new()
        } else {
            self.shell.contacts.cancel()
        };
        if !focused && let Some(cancel) = self.shell.mouse.cancel() {
            events.push(crate::ShellEvent::Pointer(cancel));
        }
        for event in &mut events {
            *event = self.input(event.clone());
        }
        events.push(crate::ShellEvent::Focused(focused));
        events
    }

    /// Assistive technology asks for `action` on the accessibility node
    /// `node` (with `value`): the event the window delivers, checked
    /// against its accepted tree as a native window checks it (`None`: the
    /// window drops the request, because the node is absent, it or an
    /// ancestor is hidden or disabled, or it does not declare the action).
    /// `Activate`, `Toggle` and `Select` all arrive as `Activate` (the
    /// platform has one default action).
    pub fn accessibility_action(
        &self,
        node: u64,
        action: kivact_types::A11yAction,
        value: Option<&str>,
    ) -> Option<crate::ShellEvent> {
        self.accessibility().request_action(node, action, value)
    }

    /// The platform hides (`true`) or shows the window, as when it is
    /// covered or minimized: a hidden window has no buffer for frames, so
    /// presenting skips them ([`Presented::Skipped`]).
    pub fn set_occluded(&mut self, occluded: bool) {
        self.shell.occluded = occluded;
    }

    /// The next `count` presentations fail as a lost surface does (the
    /// window makes its surface again: [`Presented::SurfaceLost`], and the
    /// third failure in a row is an error).
    pub fn fail_presents(&mut self, count: u32) {
        self.shell.failing = count;
    }

    /// Move the virtual clock to `ns` (never backwards).
    pub fn set_now(&mut self, ns: u64) {
        self.shell.now = self.shell.now.max(ns);
    }

    /// Resize the content to `size` logical pixels (deliver
    /// [`crate::ShellEvent::Resized`] after it).
    pub fn resize(&mut self, size: Size) {
        self.shell.size = size;
        self.resized();
    }

    /// Change the scale (deliver [`crate::ShellEvent::Resized`] after it).
    pub fn set_scale(&mut self, scale: f32) {
        self.shell.scale = scale;
        self.resized();
    }

    fn resized(&self) {
        if let Some(real) = &self.shell.real {
            let (width, height) = self.physical();
            real.resize(width, height, f64::from(self.shell.scale));
        }
    }

    /// Move to a display refreshing every `period_ns` ns (deliver
    /// [`crate::ShellEvent::EnvironmentChanged`] after it).
    pub fn set_frame_period(&mut self, period_ns: u64) {
        self.shell.period = period_ns.max(1);
    }

    /// Change the system's reduce-motion preference (deliver
    /// [`crate::ShellEvent::EnvironmentChanged`] after it).
    pub fn set_reduced_motion(&mut self, reduce: bool) {
        self.shell.reduced_motion = reduce;
    }

    /// Change the system's theme preference (deliver
    /// [`crate::ShellEvent::EnvironmentChanged`] after it).
    pub fn set_dark(&mut self, dark: bool) {
        self.shell.dark = dark;
    }

    /// Change the system's locale (deliver
    /// [`crate::ShellEvent::EnvironmentChanged`] after it).
    pub fn set_locale(&mut self, locale: Option<&str>) {
        self.shell.locale = locale.map(str::to_owned);
    }

    /// The platform destroys the window's surface, as Android does while
    /// the application is in the background (deliver
    /// [`crate::ShellEvent::SurfaceDestroyed`] after it): nothing is
    /// presented until [`VirtualWindow::create_surface`].
    pub fn destroy_surface(&mut self) {
        if let TestSurface::Live { generation } = self.shell.surface {
            self.shell.surface = TestSurface::Destroyed { last: generation };
            self.presenter.surface_destroyed();
        }
    }

    /// The platform gives the window a new surface, the next generation
    /// (deliver [`crate::ShellEvent::SurfaceCreated`] after it).
    pub fn create_surface(&mut self) {
        if let TestSurface::Destroyed { last } = self.shell.surface {
            let generation = last
                .checked_add(1)
                .expect("invariant: surface generations fit u64");
            self.shell.surface = TestSurface::Live { generation };
        }
    }

    /// Change the safe area, as a phone's system UI does when it rotates
    /// (deliver [`crate::ShellEvent::EnvironmentChanged`] after it).
    pub fn set_safe_area(&mut self, insets: Insets) {
        self.shell.safe_area = insets;
    }

    /// The in-memory clipboard.
    pub fn clipboard(&self) -> &str {
        &self.shell.clipboard
    }

    /// Put `text` on the in-memory clipboard.
    pub fn set_clipboard(&mut self, text: &str) {
        text.clone_into(&mut self.shell.clipboard);
    }

    /// Script the answer of the next file, save or folder dialog (`None`:
    /// cancelled). An unscripted dialog is cancelled.
    pub fn push_file_dialog(&mut self, path: Option<PathBuf>) {
        self.shell.dialogs.push_back(path.map(|path| vec![path]));
    }

    /// Script the next multiple-file dialog response; `None` cancels it.
    pub fn push_files_dialog(&mut self, paths: Option<Vec<PathBuf>>) {
        self.shell.dialogs.push_back(paths);
    }

    /// The paths [`Window::open_path`] was asked to open, in order (a
    /// failed request is not recorded). Nothing is started and the file
    /// system is not read.
    pub fn opened_paths(&self) -> &[PathBuf] {
        &self.shell.opened
    }

    /// The paths [`Window::reveal_path`] was asked to reveal, in order,
    /// recorded like [`VirtualWindow::opened_paths`].
    pub fn revealed_paths(&self) -> &[PathBuf] {
        &self.shell.revealed
    }

    /// Make the next clipboard, dialog, open or reveal operation return an
    /// error with `reason`, without changing the clipboard, consuming a
    /// dialog answer or recording a path. Other window operations leave
    /// the failure pending.
    pub fn fail_next_effect(&mut self, reason: &str) {
        self.shell.fail_next = Some(ShellError::new(reason));
    }

    /// [`VirtualWindow::fail_next_effect`] with the error a platform
    /// without the service answers ([`ShellError::unsupported`]), as the
    /// native window does for opening a path on macOS.
    pub fn fail_next_effect_unsupported(&mut self, reason: &str) {
        self.shell.fail_next = Some(ShellError::unsupported(reason));
    }

    /// Whether a frame was asked for and not drawn yet.
    pub fn redraw_requested(&self) -> bool {
        self.shell.redraw.get()
    }

    /// Take the request for a frame: whether there was one.
    pub fn take_redraw(&self) -> bool {
        self.shell.redraw.replace(false)
    }

    /// The title the host set.
    pub fn title(&self) -> String {
        self.shell.title.borrow().clone()
    }

    /// The minimum content size the host set (`None`: no minimum).
    pub fn min_size(&self) -> Option<Size> {
        self.shell.min_size.get()
    }

    /// The IME caret area (logical) while a text input has the focus.
    pub fn ime_area(&self) -> Option<Rect> {
        self.shell.ime.get()
    }

    /// What the host gave the window for the IME's queries.
    pub fn ime_text(&self) -> Option<ShellImeText> {
        self.shell.ime_text.borrow().clone()
    }

    /// Whether the software keyboard is shown (the host asked for it), and
    /// how often it was shown or hidden.
    pub fn soft_keyboard(&self) -> (bool, u32) {
        (
            self.shell.soft_keyboard.0.get(),
            self.shell.soft_keyboard.1.get(),
        )
    }

    /// The cursor shown over the content.
    pub fn cursor(&self) -> Cursor {
        self.shell.cursor.get()
    }

    /// The last frame presented: its physical width, height and 0x00RRGGBB
    /// pixels, row by row.
    pub fn frame(&self) -> Option<(u32, u32, &[u32])> {
        self.shell
            .frame
            .as_ref()
            .map(|(width, height, pixels)| (*width, *height, &pixels[..]))
    }

    /// The rasterizer that drew the last frame.
    pub fn renderer(&self) -> Option<Renderer> {
        self.renderer
    }

    /// The presenter's notes since the last call, in order: the renderer's
    /// changes (a GPU window's device, [`RendererChange::Gpu`]) and the
    /// external frames it could not draw, as a native window's observer
    /// ([`crate::Shell::on_renderer_change`]) gets them.
    pub fn take_renderer_changes(&self) -> Vec<RendererChange> {
        std::mem::take(
            &mut *self
                .changes
                .lock()
                .expect("renderer change log lock poisoned: a panic already happened"),
        )
    }

    /// The accessibility tree: the real window's, else the headless one.
    pub fn accessibility(&self) -> &Accessibility {
        self.shell.accessibility()
    }

    /// Whether frames are mirrored to a real window.
    pub fn has_window(&self) -> bool {
        self.shell.real.is_some()
    }

    /// Wait until the real window shows everything presented so far.
    pub fn sync(&self) {
        if let Some(real) = &self.shell.real {
            real.sync();
        }
    }

    fn physical(&self) -> (u32, u32) {
        physical(self.shell.size, self.shell.scale)
    }
}

/// The real window `options` ask for, if any.
fn open_real(
    options: &VirtualWindowOptions,
    env: &Env,
    (width, height): (u32, u32),
) -> Result<Option<TestWindow>, Error> {
    let mode = match options.mode {
        WindowMode::Auto if env.test_headless => WindowMode::Headless,
        mode => mode,
    };
    if mode == WindowMode::Headless {
        return Ok(None);
    }
    let opened = TestDisplay::global().and_then(|display| {
        display.open(TestWindowOptions {
            title: options.title.clone(),
            width,
            height,
            scale: f64::from(options.scale),
        })
    });
    match opened {
        Ok(window) => Ok(Some(window)),
        Err(e) if mode == WindowMode::Real => Err(Error::new(e.to_string())),
        Err(_) => Ok(None),
    }
}

impl Window for VirtualWindow {
    fn request_size(&mut self, size: Size) -> Result<(), ShellError> {
        self.shell.request_size(size)
    }
    fn size(&self) -> Size {
        self.shell.size()
    }
    fn scale(&self) -> f32 {
        self.shell.scale()
    }
    fn now_ns(&self) -> u64 {
        self.shell.now_ns()
    }
    fn environment(&self) -> Environment {
        self.shell.environment()
    }
    /// A test window stands for any platform: it delivers what a test
    /// injects (touch, pen, safe area, memory warnings, the IME's
    /// reconversion) and records what it is asked for (the software
    /// keyboard); assistive technology reads it when it is a real window.
    fn capabilities(&self) -> Capabilities {
        self.shell.capabilities()
    }
    fn surface_generation(&self) -> Option<u64> {
        self.shell.surface_generation()
    }
    fn request_redraw(&self) {
        self.shell.request_redraw()
    }
    fn set_fullscreen(&self, fullscreen: bool) -> Result<(), ShellError> {
        self.shell.set_fullscreen(fullscreen)
    }
    fn fullscreen(&self) -> bool {
        self.shell.fullscreen()
    }
    fn set_title(&self, title: &str) {
        self.shell.set_title(title)
    }
    fn set_min_size(&self, size: Option<Size>) {
        self.shell.set_min_size(size)
    }
    fn set_ime_allowed(&self, allowed: bool) {
        self.shell.set_ime_allowed(allowed)
    }
    fn set_ime_cursor_area(&self, area: Rect) {
        self.shell.set_ime_cursor_area(area)
    }
    fn set_ime_text(&self, text: Option<ShellImeText>) {
        self.shell.set_ime_text(text)
    }
    fn set_soft_keyboard(&self, shown: bool) {
        self.shell.set_soft_keyboard(shown)
    }
    fn set_cursor(&self, cursor: Cursor) {
        self.shell.set_cursor(cursor)
    }
    fn release_caches(&mut self) {
        self.presenter.release_caches();
    }
    fn frame_plan(&self) -> FramePlan {
        FramePlan::new(self.physical(), self.shell.scale)
    }
    fn present(&mut self, scene: &Scene, plan: FramePlan) -> Result<Presented, Error> {
        if self.shell.surface.generation().is_none() {
            return Ok(crate::presenter::no_surface());
        }
        if let Some(dropped) = crate::presenter::no_pixels(self.physical()) {
            return Ok(dropped);
        }
        if self.shell.occluded {
            return self
                .presenter
                .refused(None, self.shell.now, self.shell.period);
        }
        if self.shell.failing > 0 {
            self.shell.failing -= 1;
            let lost = Error::with(ErrorKind::Surface, "the window's surface was lost");
            return self
                .presenter
                .refused(Some(lost), self.shell.now, self.shell.period);
        }
        let (width, height) = self.physical();
        let gpu = self.shell.real.is_some() && self.shell.gpu;
        let clock = (self.shell.now, self.shell.period);
        let (presented, frame) = self.presenter.present_test(
            &mut self.shell,
            gpu,
            scene,
            plan,
            (width, height),
            clock,
        )?;
        if let Some((frame, renderer)) = frame {
            // A CPU frame was offered to the shell, which kept it; a GPU
            // frame was read back from the real window.
            if gpu {
                let pixels = crate::presenter::surface_pixels(&frame, width, height);
                self.shell.frame = Some((width, height, pixels.into()));
            }
            self.renderer = Some(renderer);
        }
        Ok(presented)
    }
    fn accessibility_active(&self) -> bool {
        self.shell.accessibility_active()
    }
    fn defers_accessibility(&self) -> bool {
        false
    }
    fn replace_accessibility(&mut self, tree: &SemanticsTree) -> Result<(), AccessibilityError> {
        self.shell.replace_accessibility(tree)
    }
    fn update_accessibility(&mut self, update: &SemanticsUpdate) -> Result<(), AccessibilityError> {
        self.shell.update_accessibility(update)
    }
    fn copy_text(&mut self, text: &str) -> Result<(), ShellError> {
        self.shell.copy_text(text)
    }
    fn read_text(&mut self) -> Result<String, ShellError> {
        self.shell.read_text()
    }
    fn pick_file(&mut self, request: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.shell.pick_file(request)
    }
    fn pick_files(
        &mut self,
        request: &ShellFileDialog,
    ) -> Result<Option<Vec<PathBuf>>, ShellError> {
        self.shell.pick_files(request)
    }
    fn save_file(&mut self, request: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.shell.save_file(request)
    }
    fn pick_folder(&mut self, request: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.shell.pick_folder(request)
    }
    fn open_path(&mut self, path: &Path) -> Result<(), ShellError> {
        self.shell.open_path(path)
    }
    fn reveal_path(&mut self, path: &Path) -> Result<(), ShellError> {
        self.shell.reveal_path(path)
    }
    #[cfg(feature = "gpu-interop")]
    fn gpu_adapter(&self) -> Option<GpuAdapter> {
        self.presenter.gpu_adapter()
    }
    #[cfg(feature = "gpu-interop")]
    fn import_external_image(
        &mut self,
        id: ExternalImageId,
        textures: SharedTextures,
    ) -> Result<(), Error> {
        self.presenter.import_external(id, textures)
    }
    #[cfg(feature = "gpu-interop")]
    fn drop_external_image(&mut self, id: ExternalImageId) {
        self.presenter.drop_external(id);
    }
}
