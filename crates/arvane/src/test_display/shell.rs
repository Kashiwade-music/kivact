//! Role: In-memory shell services with an optional native display mirror.
//!
//! This backend owns no renderer. The facade alone owns Presenter and capture
//! policy; CPU frames and native GPU targets cross the ordinary shell SPI.

use arvane_native::test_display::TestWindow;
use arvane_shell::{
    Accessibility, AccessibilityError, ShellError, ShellFileDialog, ShellImeText, ShellWindow,
};
use kivact_types::protocol::{Environment, MouseSettings, SemanticsTree, SemanticsUpdate};
use kivact_types::{Capabilities, Cursor, Insets, Rect, Size};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct VirtualShell {
    pub(super) now: u64,
    pub(super) size: Size,
    pub(super) requested_sizes: Vec<Size>,
    pub(super) scale: f32,
    /// The display's refresh period (ns).
    pub(super) period: u64,
    pub(super) reduced_motion: bool,
    pub(super) dark: bool,
    /// The system's locale the test gives the window.
    pub(super) locale: Option<String>,
    /// The safe area the test gives the window.
    pub(super) safe_area: Insets,
    pub(super) clipboard: String,
    pub(super) dialogs: VecDeque<Option<Vec<PathBuf>>>,
    /// Paths asked to be opened, in order.
    pub(super) opened: Vec<PathBuf>,
    /// Paths asked to be revealed, in order.
    pub(super) revealed: Vec<PathBuf>,
    pub(super) fail_next: Option<ShellError>,
    pub(super) redraw: Cell<bool>,
    pub(super) title: RefCell<String>,
    pub(super) min_size: Cell<Option<Size>>,
    /// The IME caret area while a text input has the focus.
    pub(super) ime: Cell<Option<Rect>>,
    /// What the focused text input shows the IME.
    pub(super) ime_text: RefCell<Option<ShellImeText>>,
    /// The cursor shown over the content.
    pub(super) cursor: Cell<Cursor>,
    pub(super) fullscreen: Cell<bool>,
    /// The software keyboard is shown; how often it was shown or hidden.
    pub(super) soft_keyboard: (Cell<bool>, Cell<u32>),
    pub(super) frame: Option<(u32, u32, Arc<[u32]>)>,
    /// Frames are drawn on the GPU.
    pub(super) gpu: bool,
    pub(super) real: Option<TestWindow>,
    /// The accessibility tree without a real window.
    pub(super) headless: Accessibility,
    /// Counts the presses injected (`input`).
    pub(super) clicks: arvane_shell::ClickCounter,
    /// The mouse's position and held buttons (`mouse`, `input`).
    pub(super) mouse: arvane_shell::Mouse,
    /// The touch and pen contacts injected (`contact`).
    pub(super) contacts: arvane_shell::Contacts,
    /// The platform hides the window: it has no buffer for frames.
    pub(super) occluded: bool,
    /// Presentations that are to fail as a lost surface does.
    pub(super) failing: u32,
    /// The window's surface.
    pub(super) surface: TestSurface,
    pub(super) cpu_frame: Cell<bool>,
}

/// The window's presentation surface, as the test drives the platform.
#[derive(Clone, Copy, Debug)]
pub(super) enum TestSurface {
    /// The surface of `generation` (the count of surfaces made; the first is 1).
    Live { generation: NonZeroU64 },
    /// Destroyed by the platform; the last one was of generation `last`.
    Destroyed { last: NonZeroU64 },
}

impl TestSurface {
    /// The live surface's generation; `None` while destroyed.
    pub(super) fn generation(self) -> Option<u64> {
        match self {
            Self::Live { generation } => Some(generation.get()),
            Self::Destroyed { .. } => None,
        }
    }
}

impl VirtualShell {
    /// Consume a scripted service failure through the service's Result,
    /// just as a native service reports an OS failure.
    fn effect_result(&mut self) -> Result<(), ShellError> {
        match self.fail_next.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
    /// The accessibility tree: the real window's, else the headless one.
    pub(super) fn accessibility(&self) -> &Accessibility {
        self.real
            .as_ref()
            .map_or(&self.headless, TestWindow::accessibility)
    }
}

impl ShellWindow for VirtualShell {
    fn request_size(&mut self, size: Size) -> Result<(), ShellError> {
        self.effect_result()?;
        self.requested_sizes.push(size);
        Ok(())
    }
    fn wait_until(&mut self, done: &mut dyn FnMut() -> bool, limit: std::time::Duration) -> bool {
        let started = std::time::Instant::now();
        while !done() {
            if started.elapsed() >= limit {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        true
    }

    fn copy_text(&mut self, text: &str) -> Result<(), ShellError> {
        self.effect_result()?;
        text.clone_into(&mut self.clipboard);
        Ok(())
    }
    fn read_text(&mut self) -> Result<String, ShellError> {
        self.effect_result()?;
        Ok(self.clipboard.clone())
    }
    fn pick_file(&mut self, _: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.effect_result()?;
        Ok(self
            .dialogs
            .pop_front()
            .flatten()
            .and_then(|paths| paths.into_iter().next()))
    }
    fn pick_files(&mut self, _: &ShellFileDialog) -> Result<Option<Vec<PathBuf>>, ShellError> {
        self.effect_result()?;
        Ok(self.dialogs.pop_front().flatten())
    }
    fn save_file(&mut self, _: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.effect_result()?;
        Ok(self
            .dialogs
            .pop_front()
            .flatten()
            .and_then(|paths| paths.into_iter().next()))
    }
    fn pick_folder(&mut self, _: &ShellFileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.effect_result()?;
        Ok(self
            .dialogs
            .pop_front()
            .flatten()
            .and_then(|paths| paths.into_iter().next()))
    }
    fn open_path(&mut self, path: &Path) -> Result<(), ShellError> {
        self.effect_result()?;
        self.opened.push(path.to_path_buf());
        Ok(())
    }
    fn reveal_path(&mut self, path: &Path) -> Result<(), ShellError> {
        self.effect_result()?;
        self.revealed.push(path.to_path_buf());
        Ok(())
    }
    fn replace_accessibility(&mut self, tree: &SemanticsTree) -> Result<(), AccessibilityError> {
        self.accessibility().replace(tree)
    }
    fn update_accessibility(&mut self, update: &SemanticsUpdate) -> Result<(), AccessibilityError> {
        self.accessibility().update(update)
    }
    fn accessibility_active(&self) -> bool {
        true
    }
    fn size(&self) -> Size {
        self.size
    }
    fn scale(&self) -> f32 {
        self.scale
    }
    fn now_ns(&self) -> u64 {
        self.now
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            touch: true,
            pen: true,
            soft_keyboard: true,
            safe_area: true,
            memory_warnings: true,
            ime_reconversion: true,
            native_accessibility: self.real.is_some(),
            gpu: self.gpu,
        }
    }
    fn surface_generation(&self) -> Option<u64> {
        self.surface.generation()
    }
    fn request_redraw(&self) {
        self.redraw.set(true);
    }
    fn set_title(&self, title: &str) {
        title.clone_into(&mut self.title.borrow_mut());
    }
    fn set_min_size(&self, size: Option<Size>) {
        self.min_size.set(size);
    }
    fn environment(&self) -> Environment {
        let (double_click_ns, double_click_distance) = arvane_shell::DEFAULT_DOUBLE_CLICK;
        Environment {
            scale: self.scale,
            frame_period_ns: self.period,
            reduced_motion: self.reduced_motion,
            dark: self.dark,
            safe_area: self.safe_area,
            mouse: MouseSettings {
                double_click_ns,
                double_click_distance,
                ..MouseSettings::default()
            },
            locale: self.locale.clone(),
        }
    }
    fn set_fullscreen(&self, fullscreen: bool) -> Result<(), ShellError> {
        self.fullscreen.set(fullscreen);
        Ok(())
    }
    fn fullscreen(&self) -> bool {
        self.fullscreen.get()
    }
    fn set_cursor(&self, cursor: Cursor) {
        self.cursor.set(cursor);
    }
    fn set_ime_allowed(&self, allowed: bool) {
        if !allowed {
            self.ime.set(None);
        }
    }
    fn set_soft_keyboard(&self, shown: bool) {
        if self.soft_keyboard.0.replace(shown) != shown {
            self.soft_keyboard.1.set(self.soft_keyboard.1.get() + 1);
        }
    }
    fn set_ime_cursor_area(&self, area: Rect) {
        self.ime.set(Some(area));
    }
    fn set_ime_text(&self, text: Option<ShellImeText>) {
        *self.ime_text.borrow_mut() = text;
    }
    fn key(&self) -> u64 {
        self.real.as_ref().map_or(0, TestWindow::key)
    }
    fn physical_size(&self) -> (u32, u32) {
        physical(self.size, self.scale)
    }
    fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }
    fn frame_period_ns(&self) -> u64 {
        self.period
    }
    fn recover_surface(&mut self) -> Result<(), ShellError> {
        Ok(())
    }
    fn shows_cpu_frame(&self) -> bool {
        self.cpu_frame.get()
    }
    fn release_cpu_surface(&mut self) {
        self.cpu_frame.set(false);
        if let Some(real) = &self.real {
            real.gpu_presents();
        }
    }
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError> {
        match &self.real {
            Some(real) => real.with_gpu_target(make),
            None => Err(ShellError::unsupported(
                "the headless shell has no native GPU target",
            )),
        }
    }
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        draw: &mut dyn FnMut(&mut [u32]) -> bool,
    ) -> Result<bool, ShellError> {
        if self.surface.generation().is_none() || width == 0 || height == 0 {
            return Ok(false);
        }
        if (width, height) != self.physical_size() {
            return Err(ShellError::new("frame size differs from window surface"));
        }
        let mut pixels = vec![0; width as usize * height as usize];
        if !draw(&mut pixels) {
            return Ok(false);
        }
        let pixels: Arc<[u32]> = pixels.into();
        if let Some(real) = &self.real {
            real.present(width, height, pixels.clone());
        }
        self.frame = Some((width, height, pixels));
        self.cpu_frame.set(true);
        Ok(true)
    }
}

pub(super) fn physical(size: Size, scale: f32) -> (u32, u32) {
    let side = |v: f32| (v * scale).round().max(0.0) as u32;
    (side(size.width), side(size.height))
}
