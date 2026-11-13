//! Role: In-memory shell services with an optional native display mirror.
//!
//! This backend owns no renderer. The facade alone owns Presenter and capture
//! policy; CPU frames and native GPU targets cross the ordinary shell SPI.

use crate::clock::Clock;
use crate::script::Responder;
use crate::surface::TestSurface;
use arvane_shell::{
    Accessibility, AccessibilityError, BufferAge, ShellError, ShellImeText, ShellWindow,
};
use kivact_types::Environment;
use kivact_types::render::{PixelRect, SemanticsTree, SemanticsUpdate};
use kivact_types::{Capabilities, Cursor, InputPurpose, Rect, Size, WindowRequest};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::rc::Rc;
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
    /// The request of the main window (`WindowId::MAIN`).
    pub main: WindowRequest,
    pub scale: f64,
    pub frame_period_ns: u64,
    pub reduced_motion: bool,
    pub dark: bool,
    pub locale: Option<String>,
    pub gpu: GpuMode,
    pub mirror: MirrorMode,
    /// Advertised service capabilities; defaults match the virtual backend's built-in services.
    /// False flags do not disable those services or scripted responders. GPU and native
    /// accessibility are always reported from the actual backend, overriding these flags.
    pub capabilities: Capabilities,
    /// Restoration snapshot for this launch.
    pub restored_state: BTreeMap<String, Vec<u8>>,
}
impl Default for VirtualOptions {
    fn default() -> Self {
        Self {
            main: WindowRequest::main(
                String::new(),
                Size {
                    width: 800.0,
                    height: 600.0,
                },
            ),
            scale: 1.0,
            frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
            reduced_motion: false,
            dark: false,
            locale: None,
            gpu: GpuMode::Off,
            mirror: MirrorMode::Auto,
            capabilities: Capabilities {
                state_restoration: true,
                clipboard_text: true,
                file_dialogs: true,
                open_path: true,
                reveal_path: true,
                fullscreen: true,
                min_size: true,
                touch: true,
                pen: true,
                soft_keyboard: true,
                safe_area: true,
                memory_warnings: true,
                ime_reconversion: true,
                multi_window: true,
                window_placement: true,
                ..Capabilities::default()
            },
            restored_state: Default::default(),
        }
    }
}

/// A frame the window keeps: physical size and XRGB pixels.
pub(crate) struct Kept {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Arc<[u32]>,
}

/// What the window shows of the frames presented to it.
pub(crate) enum Shown {
    Nothing,
    /// The last CPU presentation, still on the CPU surface: a damaged presentation starts from it.
    OnCpuSurface(Kept),
    /// A kept frame: a GPU frame read back, or the last CPU frame after the CPU surface was released.
    Kept(Kept),
}

/// Where the window's accessibility tree and frames go.
pub(crate) enum Display {
    Headless(Accessibility),
    Mirrored(Box<dyn Mirror>),
}

/// The state of the whole virtual loop: the services every window shares.
pub(crate) struct VirtualPlatform {
    /// Saved application bytes, shared with the script that supplies them.
    pub(crate) saved_state: Rc<RefCell<BTreeMap<String, Vec<u8>>>>,
    /// Scripted service overrides; the newest responder returning a result wins.
    pub(crate) responders: RefCell<Vec<Responder>>,
    /// The loop clock, synced from the script clock at each step.
    pub(crate) now: Clock,
    /// Immutable advertised service configuration supplied at construction.
    pub(crate) capabilities: Capabilities,
    /// Clipboard text set through a service request.
    pub(crate) clipboard: RefCell<String>,
    pub(crate) dialogs: RefCell<VecDeque<Option<Vec<PathBuf>>>>,
    /// Paths asked to be opened, in order.
    pub(crate) opened: RefCell<Vec<PathBuf>>,
    /// Paths asked to be revealed, in order.
    pub(crate) revealed: RefCell<Vec<PathBuf>>,
    // xtask-allow(state-fields): option: the service failure the test fills and the window drains; absent when none is pending
    /// The service failure the next effect consumes.
    pub(crate) fail_next: RefCell<Option<ShellError>>,
}

/// One virtual window: the state of a single window of the loop.
pub(crate) struct VirtualWindow {
    /// The loop state this window's requests go to.
    pub(crate) platform: Rc<VirtualPlatform>,
    /// The single snapshot of the window's environment and preferences.
    pub(crate) env: RefCell<Environment>,
    pub(crate) requested_sizes: Vec<Size>,
    // xtask-allow(state-fields): independent: the script's redraw request is recorded separately; every combination is a valid script state
    /// A redraw was requested and not delivered yet.
    pub(crate) redraw: Cell<bool>,
    pub(crate) title: RefCell<String>,
    // xtask-allow(state-fields): independent: the size the application sets is recorded separately; every combination is a valid script state
    /// The minimum content size the application set.
    pub(crate) min_size: Cell<Option<Size>>,
    /// The content purpose the focused text input set.
    pub(crate) ime_purpose: Cell<InputPurpose>,
    // xtask-allow(state-fields): independent: the script's IME caret area is recorded separately; every combination is a valid script state
    /// The IME caret area while a text input has the focus.
    pub(crate) ime: Cell<Option<Rect>>,
    // xtask-allow(state-fields): independent: the script's surrounding text is recorded separately; every combination is a valid script state
    /// What the focused text input shows the IME.
    pub(crate) ime_text: RefCell<Option<ShellImeText>>,
    /// The cursor shown over the content.
    pub(crate) cursor: Cell<Cursor>,
    // xtask-allow(state-fields): independent: the script's soft keyboard request is recorded separately; every combination is a valid script state
    /// The software keyboard is shown.
    pub(crate) soft_keyboard_shown: Cell<bool>,
    // xtask-allow(state-fields): statistic: soft keyboard requests counted for tests
    /// How often the software keyboard was shown or hidden.
    pub(crate) soft_keyboard_requests: Cell<u32>,
    /// What the window shows of the frames presented to it.
    pub(crate) shown: Shown,
    // xtask-allow(state-fields): independent: the renderer the script requires is recorded separately; every combination is a valid script state
    /// Frames are drawn on the GPU.
    pub(crate) gpu: bool,
    /// Where the window's accessibility tree and frames go.
    pub(crate) display: Display,
    // xtask-allow(state-fields): independent: the script's occlusion is recorded separately; every combination is a valid script state
    /// The platform hides the window: it has no buffer for frames.
    pub(crate) occluded: bool,
    /// Presentations that are to fail as a lost surface does.
    pub(crate) failing: u32,
    /// The window's surface.
    pub(crate) surface: TestSurface,
    // xtask-allow(state-fields): statistic: focus requests counted for tests
    /// How often the application asked to bring this window to the front.
    pub(crate) focus_requests: Cell<u32>,
    /// Key presses nobody handled since a test last read them.
    pub(crate) unhandled_keys: RefCell<Vec<kivact_types::KeyCode>>,
}

impl VirtualWindow {
    /// Build the window `request` describes, with `display` already decided.
    pub(crate) fn new(
        platform: &Rc<VirtualPlatform>,
        options: &VirtualOptions,
        request: &WindowRequest,
        display: Display,
    ) -> VirtualWindow {
        VirtualWindow {
            platform: Rc::clone(platform),
            env: RefCell::new(Environment {
                window_size: request.size,
                scale_factor: options.scale,
                frame_period_ns: options.frame_period_ns.max(1),
                reduced_motion: options.reduced_motion,
                theme_hint: if options.dark {
                    kivact_types::ThemeHint::Dark
                } else {
                    kivact_types::ThemeHint::Light
                },
                locale: options.locale.clone(),
                restored_state: options.restored_state.clone(),
                ..Environment::default()
            }),
            requested_sizes: Vec::new(),
            redraw: Cell::new(false),
            title: RefCell::new(request.title.clone()),
            min_size: Cell::new(request.min_size),
            ime_purpose: Cell::new(InputPurpose::Text),
            ime: Cell::new(None),
            ime_text: RefCell::new(None),
            cursor: Cell::new(Cursor::Default),
            soft_keyboard_shown: Cell::new(false),
            soft_keyboard_requests: Cell::new(0),
            shown: Shown::Nothing,
            gpu: options.gpu == GpuMode::Required,
            display,
            occluded: false,
            failing: 0,
            surface: TestSurface::Live {
                generation: std::num::NonZeroU64::MIN,
            },
            focus_requests: Cell::new(0),
            unhandled_keys: RefCell::new(Vec::new()),
        }
    }

    /// Consume a scripted service failure through the service's Result,
    /// just as a native service reports an OS failure.
    fn effect_result(&self) -> Result<(), ShellError> {
        match self.platform.fail_next.borrow_mut().take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// The accessibility tree: the real window's, else the headless one.
    pub(crate) fn accessibility(&self) -> &Accessibility {
        match &self.display {
            Display::Headless(tree) => tree,
            Display::Mirrored(real) => real.accessibility(),
        }
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

impl ShellWindow for VirtualWindow {
    fn perform(&mut self, request: kivact_types::PlatformRequest) -> kivact_types::PlatformResult {
        use kivact_types::{PlatformRequest, PlatformResult};
        for responder in self.platform.responders.borrow_mut().iter_mut().rev() {
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
        if let PlatformRequest::SaveState { key, bytes } = request {
            self.platform
                .saved_state
                .borrow_mut()
                .insert(key.to_string(), bytes);
            return PlatformResult::Done;
        }
        if let PlatformRequest::CopyText(text) = request {
            *self.platform.clipboard.borrow_mut() = text;
            return PlatformResult::Done;
        }
        if let PlatformRequest::ReadText = request {
            return PlatformResult::Text(self.platform.clipboard.borrow().clone());
        }
        if let PlatformRequest::PickFile(_)
        | PlatformRequest::SaveFile(_)
        | PlatformRequest::PickFolder(_) = request
        {
            return PlatformResult::Path(
                self.platform
                    .dialogs
                    .borrow_mut()
                    .pop_front()
                    .flatten()
                    .and_then(|paths| paths.into_iter().next()),
            );
        }
        if let PlatformRequest::PickFiles(_) = request {
            return PlatformResult::Paths(self.platform.dialogs.borrow_mut().pop_front().flatten());
        }
        if let PlatformRequest::OpenPath(path) = request {
            self.platform.opened.borrow_mut().push(path);
            return PlatformResult::Done;
        }
        if let PlatformRequest::RevealPath(path) = request {
            self.platform.revealed.borrow_mut().push(path);
            return PlatformResult::Done;
        }
        if let PlatformRequest::FocusWindow = request {
            return self.focus().map(|()| PlatformResult::Done).unwrap_or_else(
                |error| match error {
                    ShellError::Unsupported(_) => PlatformResult::Unsupported,
                    ShellError::Failed(failure) => PlatformResult::Failed(failure),
                },
            );
        }
        if let PlatformRequest::RequestWindowSize(size) = request {
            self.requested_sizes.push(size);
            return PlatformResult::Done;
        }
        if let PlatformRequest::Fullscreen(fullscreen) = request {
            self.env.borrow_mut().fullscreen = fullscreen;
            return PlatformResult::Done;
        }
        PlatformResult::Unsupported
    }

    fn focus(&mut self) -> Result<(), ShellError> {
        self.focus_requests.set(self.focus_requests.get() + 1);
        Ok(())
    }

    fn cpu_format(&self) -> arvane_shell::CpuFormat {
        arvane_shell::CpuFormat::Xrgb8888
    }

    fn keeps_frames(&self) -> bool {
        true
    }
    fn keep_frame(&mut self, frame: kivact_types::render::CpuFrame) {
        self.shown = Shown::Kept(Kept {
            width: frame.width,
            height: frame.height,
            pixels: frame.to_softbuffer().into(),
        });
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
        self.platform.now.now()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            native_accessibility: matches!(self.display, Display::Mirrored(_)),
            gpu: self.gpu,
            ..self.platform.capabilities
        }
    }
    fn surface_generation(&self) -> Option<u64> {
        self.surface.generation()
    }
    fn unhandled_key(&mut self, code: kivact_types::KeyCode) {
        self.unhandled_keys.borrow_mut().push(code);
    }
    fn request_redraw(&self) {
        self.redraw.set(true);
    }
    fn set_title(&self, title: &str) {
        title.clone_into(&mut self.title.borrow_mut());
        if let Display::Mirrored(real) = &self.display {
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
    fn set_ime_purpose(&self, purpose: InputPurpose) {
        self.ime_purpose.set(purpose);
    }
    fn set_ime_allowed(&self, allowed: bool) {
        if !allowed {
            self.ime.set(None);
        }
    }
    fn set_soft_keyboard(&self, shown: bool) {
        if self.soft_keyboard_shown.replace(shown) != shown {
            self.soft_keyboard_requests
                .set(self.soft_keyboard_requests.get() + 1);
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
        matches!(self.shown, Shown::OnCpuSurface(_))
    }
    fn release_cpu_surface(&mut self) {
        self.shown = match std::mem::replace(&mut self.shown, Shown::Nothing) {
            Shown::OnCpuSurface(kept) | Shown::Kept(kept) => Shown::Kept(kept),
            Shown::Nothing => Shown::Nothing,
        };
        if let Display::Mirrored(real) = &self.display {
            real.gpu_presents();
        }
    }
    fn with_gpu_target(
        &self,
        make: Box<dyn FnOnce(arvane_shell::GpuTarget) + Send>,
    ) -> Result<(), ShellError> {
        match &self.display {
            Display::Mirrored(real) => real.with_gpu_target(make),
            Display::Headless(_) => Err(ShellError::unsupported(
                "the headless shell has no native GPU target",
            )),
        }
    }
    fn present_with(
        &mut self,
        width: u32,
        height: u32,
        damage: Option<&[PixelRect]>,
        draw: &mut dyn FnMut(&mut [u32], BufferAge) -> bool,
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
        // With damage, the buffer is the CPU frame presented last time, taken from the surface and
        // updated in place (copied only while a mirror or a screenshot still holds it).
        // A GPU frame kept for the device-loss fallback is not that frame.
        let kept = match std::mem::replace(&mut self.shown, Shown::Nothing) {
            Shown::OnCpuSurface(kept)
                if damage.is_some() && (kept.width, kept.height) == (width, height) =>
            {
                Some(kept)
            }
            other => {
                self.shown = other;
                None
            }
        };
        let reused = kept.is_some();
        let (mut pixels, age): (Arc<[u32]>, BufferAge) = match kept {
            Some(kept) => (kept.pixels, BufferAge::Previous),
            None => (
                vec![0_u32; width as usize * height as usize].into(),
                BufferAge::Unknown,
            ),
        };
        if !draw(unique_pixels(&mut pixels), age) {
            if reused {
                self.shown = Shown::OnCpuSurface(Kept {
                    width,
                    height,
                    pixels,
                });
            }
            return Ok(false);
        }
        if let Display::Mirrored(real) = &self.display {
            real.present(width, height, pixels.clone());
        }
        self.shown = Shown::OnCpuSurface(Kept {
            width,
            height,
            pixels,
        });
        Ok(true)
    }
}

/// The pixels to write: the buffer itself when nothing else holds it, otherwise a copy of it.
fn unique_pixels(pixels: &mut Arc<[u32]>) -> &mut [u32] {
    if Arc::get_mut(pixels).is_none() {
        *pixels = Arc::from(&pixels[..]);
    }
    Arc::get_mut(pixels).expect("the buffer is unique after the copy")
}

pub(crate) fn physical(size: Size, scale: f64) -> (u32, u32) {
    let side = |v: f64| (v * scale).round().max(0.0) as u32;
    (side(size.width), side(size.height))
}
