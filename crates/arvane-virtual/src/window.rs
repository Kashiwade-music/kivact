//! Role: In-memory shell services with an optional native display mirror.
//!
//! This backend owns no renderer. The facade alone owns Presenter and capture
//! policy; CPU frames and native GPU targets cross the ordinary shell SPI.

use crate::surface::TestSurface;
use arvane_shell::{Accessibility, AccessibilityError, ShellError, ShellImeText, ShellWindow};
use kivact_types::Environment;
use kivact_types::render::{SemanticsTree, SemanticsUpdate};
use kivact_types::{Capabilities, Cursor, Rect, Size};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

/// GPU presentation policy for a virtual window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GpuMode {
    #[default]
    Off,
    Required,
}
/// Whether virtual output must be mirrored to a native window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MirrorMode {
    Off,
    #[default]
    Auto,
    Required,
}
/// Initial virtual platform environment.
#[derive(Clone, Debug)]
pub struct VirtualOptions {
    pub title: String,
    pub size: Size,
    pub scale: f64,
    pub frame_period_ns: u64,
    pub reduced_motion: bool,
    pub dark: bool,
    pub locale: Option<String>,
    pub gpu: GpuMode,
    pub mirror: MirrorMode,
    /// Restoration snapshot for this launch.
    pub restored_state: std::collections::BTreeMap<String, Vec<u8>>,
}
impl Default for VirtualOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            size: Size {
                width: 800.0,
                height: 600.0,
            },
            scale: 1.0,
            frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
            reduced_motion: false,
            dark: false,
            locale: None,
            gpu: GpuMode::Off,
            mirror: MirrorMode::Auto,
            restored_state: Default::default(),
        }
    }
}
pub(crate) struct VirtualShell {
    pub(crate) saved_state: std::rc::Rc<RefCell<std::collections::BTreeMap<String, Vec<u8>>>>,
    pub(crate) ime_purpose: Cell<kivact_types::InputPurpose>,
    pub(crate) responders: Vec<crate::script::Responder>,
    pub(crate) now: u64,
    /// The single snapshot of the window's environment and preferences.
    pub(crate) env: RefCell<Environment>,
    pub(crate) requested_sizes: Vec<Size>,
    pub(crate) clipboard: String,
    pub(crate) dialogs: VecDeque<Option<Vec<PathBuf>>>,
    /// Paths asked to be opened, in order.
    pub(crate) opened: Vec<PathBuf>,
    /// Paths asked to be revealed, in order.
    pub(crate) revealed: Vec<PathBuf>,
    pub(crate) fail_next: Option<ShellError>,
    pub(crate) redraw: Cell<bool>,
    pub(crate) title: RefCell<String>,
    pub(crate) min_size: Cell<Option<Size>>,
    /// The IME caret area while a text input has the focus.
    pub(crate) ime: Cell<Option<Rect>>,
    /// What the focused text input shows the IME.
    pub(crate) ime_text: RefCell<Option<ShellImeText>>,
    /// The cursor shown over the content.
    pub(crate) cursor: Cell<Cursor>,
    /// The software keyboard is shown; how often it was shown or hidden.
    pub(crate) soft_keyboard: (Cell<bool>, Cell<u32>),
    pub(crate) frame: Option<(u32, u32, Arc<[u32]>)>,
    /// Frames are drawn on the GPU.
    pub(crate) gpu: bool,
    pub(crate) real: Option<Box<dyn Mirror>>,
    /// The accessibility tree without a real window.
    pub(crate) headless: Accessibility,
    /// The platform hides the window: it has no buffer for frames.
    pub(crate) occluded: bool,
    /// Presentations that are to fail as a lost surface does.
    pub(crate) failing: u32,
    /// The window's surface.
    pub(crate) surface: TestSurface,
    pub(crate) cpu_frame: Cell<bool>,
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
    pub(crate) fn accessibility(&self) -> &Accessibility {
        self.real
            .as_ref()
            .map_or(&self.headless, |real| real.accessibility())
    }
}

/// A mirror accepts rendered output without contributing input.
pub(crate) trait Mirror {
    fn present(&self, width: u32, height: u32, pixels: Arc<[u32]>);
    fn accessibility(&self) -> &Accessibility;
    fn set_title(&self, title: &str);
    fn resize(&self, width: u32, height: u32, scale: f64);
    fn sync(&self);
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError>;
    fn gpu_presents(&self);
}

impl ShellWindow for VirtualShell {
    fn perform(&mut self, request: kivact_types::PlatformRequest) -> kivact_types::PlatformResult {
        use kivact_types::{PlatformRequest, PlatformResult};
        for responder in self.responders.iter_mut().rev() {
            if let Some(result) = responder(&request) {
                return result;
            }
        }
        if !matches!(request, PlatformRequest::Fullscreen(_))
            && let Err(error) = self.effect_result()
        {
            return match error {
                ShellError::Unsupported(_) => PlatformResult::Unsupported,
                ShellError::Failed(failure) => PlatformResult::Failed(failure),
            };
        }
        match request {
            PlatformRequest::SaveState { key, bytes } => {
                self.saved_state.borrow_mut().insert(key.to_string(), bytes);
                PlatformResult::Done
            }
            PlatformRequest::CopyText(text) => {
                self.clipboard = text;
                PlatformResult::Done
            }
            PlatformRequest::ReadText => PlatformResult::Text(self.clipboard.clone()),
            PlatformRequest::PickFile(_)
            | PlatformRequest::SaveFile(_)
            | PlatformRequest::PickFolder(_) => PlatformResult::Path(
                self.dialogs
                    .pop_front()
                    .flatten()
                    .and_then(|paths| paths.into_iter().next()),
            ),
            PlatformRequest::PickFiles(_) => {
                PlatformResult::Paths(self.dialogs.pop_front().flatten())
            }
            PlatformRequest::OpenPath(path) => {
                self.opened.push(path);
                PlatformResult::Done
            }
            PlatformRequest::RevealPath(path) => {
                self.revealed.push(path);
                PlatformResult::Done
            }
            PlatformRequest::RequestWindowSize(size) => {
                self.requested_sizes.push(size);
                PlatformResult::Done
            }
            PlatformRequest::Fullscreen(fullscreen) => {
                self.env.borrow_mut().fullscreen = fullscreen;
                PlatformResult::Done
            }
        }
    }
    fn cpu_format(&self) -> arvane_shell::CpuFormat {
        arvane_shell::CpuFormat::Xrgb8888
    }

    fn keeps_frames(&self) -> bool {
        true
    }
    fn keep_frame(&mut self, frame: kivact_types::render::CpuFrame) {
        self.frame = Some((frame.width, frame.height, frame.to_softbuffer().into()));
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

    fn replace_accessibility(&mut self, tree: &SemanticsTree) -> Result<(), AccessibilityError> {
        self.accessibility().replace(tree)
    }
    fn update_accessibility(&mut self, update: &SemanticsUpdate) -> Result<(), AccessibilityError> {
        self.accessibility().update(update)
    }
    fn size(&self) -> Size {
        self.env.borrow().window_size
    }
    fn scale(&self) -> f64 {
        self.env.borrow().scale_factor
    }
    fn now_ns(&self) -> u64 {
        self.now
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            state_restoration: true,
            clipboard_text: true,
            file_dialogs: true,
            open_path: true,
            reveal_path: true,
            fullscreen: true,
            min_size: true,
            command_key: kivact_types::CommandKey::Ctrl,
            context_menu_on: kivact_types::ContextMenuTrigger::Release,
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
        if let Some(real) = &self.real {
            real.set_title(title);
        }
    }
    fn set_min_size(&self, size: Option<Size>) {
        self.min_size.set(size);
    }
    fn environment(&self) -> Environment {
        self.env.borrow().clone()
    }

    fn fullscreen(&self) -> bool {
        self.env.borrow().fullscreen
    }
    fn set_cursor(&self, cursor: Cursor) {
        self.cursor.set(cursor);
    }
    fn set_ime_purpose(&self, purpose: kivact_types::InputPurpose) {
        self.ime_purpose.set(purpose);
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
    fn physical_size(&self) -> (u32, u32) {
        physical(self.size(), self.scale())
    }
    fn reduced_motion(&self) -> bool {
        self.env.borrow().reduced_motion
    }
    fn frame_period_ns(&self) -> u64 {
        self.env.borrow().frame_period_ns
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
        if self.occluded {
            return Ok(false);
        }
        if self.failing > 0 {
            self.failing -= 1;
            return Err(ShellError::new("the window's surface was lost"));
        }
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

pub(crate) fn physical(size: Size, scale: f64) -> (u32, u32) {
    let side = |v: f64| (v * scale).round().max(0.0) as u32;
    (side(size.width), side(size.height))
}
