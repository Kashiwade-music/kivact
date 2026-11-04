//! Role: One concrete rendering window and shared backend lifecycle driver.

use crate::presentation::{Notes, Observer};
use crate::presenter::Presenter;
use crate::{
    AccessibilityError, Failure, FramePlan, Presented, RendererChange, ShellControl, ShellError,
    ShellEvent, ShellImeText, ShellWaker,
};
use arvane_shell::{
    Backend, ShellHandler, ShellWindow, Step, StepOutcome, SteppedBackend, SteppedLoop,
};
use kivact_types::Environment;
use kivact_types::GpuPreference;
use kivact_types::render::{RenderScene, SemanticsTree, SemanticsUpdate};
use kivact_types::{Capabilities, Cursor, Rect, Size};
#[cfg(feature = "gpu-interop")]
use kivact_types::{ExternalImageId, GpuAdapter, SharedTextures};

/// The native window of a running [`Shell`], handed to the [`Handler`]'s
/// callbacks.
pub struct Window<'a> {
    shell: &'a mut dyn ShellWindow,
    presenter: &'a mut Presenter,
}

impl Window<'_> {
    pub fn perform(
        &mut self,
        request: kivact_types::PlatformRequest,
    ) -> kivact_types::PlatformResult {
        self.shell.perform(request)
    }

    pub fn size(&self) -> Size {
        self.shell.size()
    }
    pub fn scale(&self) -> f64 {
        self.shell.scale()
    }
    pub fn now_ns(&self) -> u64 {
        self.shell.now_ns()
    }
    pub fn environment(&self) -> Environment {
        self.shell.environment()
    }
    pub fn capabilities(&self) -> Capabilities {
        Capabilities {
            gpu: cfg!(feature = "gpu"),
            ..self.shell.capabilities()
        }
    }
    pub fn surface_generation(&self) -> Option<u64> {
        self.shell.surface_generation()
    }
    pub fn request_redraw(&self) {
        self.shell.request_redraw();
    }

    pub fn fullscreen(&self) -> bool {
        self.shell.fullscreen()
    }
    pub fn set_title(&self, title: &str) {
        self.shell.set_title(title);
    }
    pub fn set_min_size(&self, size: Option<Size>) {
        self.shell.set_min_size(size);
    }
    pub fn set_ime_allowed(&self, allowed: bool) {
        self.shell.set_ime_allowed(allowed);
    }
    pub fn set_ime_cursor_area(&self, area: Rect) {
        self.shell.set_ime_cursor_area(area);
    }
    pub fn set_ime_text(&self, text: Option<ShellImeText>) {
        self.shell.set_ime_text(text);
    }
    pub fn set_soft_keyboard(&self, shown: bool) {
        self.shell.set_soft_keyboard(shown);
    }
    pub fn set_cursor(&self, cursor: Cursor) {
        self.shell.set_cursor(cursor);
    }
    pub fn release_caches(&mut self) {
        self.presenter.release_caches();
    }
    pub fn frame_plan(&self) -> FramePlan {
        FramePlan::new(self.shell.physical_size(), self.shell.scale())
    }
    pub fn present(&mut self, scene: &RenderScene, plan: FramePlan) -> Result<Presented, Failure> {
        if self.shell.surface_generation().is_none() {
            return Ok(crate::presenter::no_surface());
        }
        if let Some(dropped) = crate::presenter::no_pixels(self.shell.physical_size()) {
            return Ok(dropped);
        }
        self.presenter.present_native(self.shell, scene, plan)
    }
    pub fn accessibility_active(&self) -> bool {
        self.shell.accessibility_active()
    }
    pub fn replace_accessibility(
        &mut self,
        tree: &SemanticsTree,
    ) -> Result<(), AccessibilityError> {
        self.shell.replace_accessibility(tree)
    }
    pub fn update_accessibility(
        &mut self,
        update: &SemanticsUpdate,
    ) -> Result<(), AccessibilityError> {
        self.shell.update_accessibility(update)
    }

    #[cfg(feature = "gpu-interop")]
    pub fn gpu_adapter(&self) -> Option<GpuAdapter> {
        self.presenter.gpu_adapter()
    }
    #[cfg(feature = "gpu-interop")]
    pub fn import_external_image(
        &mut self,
        id: ExternalImageId,
        textures: SharedTextures,
    ) -> Result<(), Failure> {
        self.presenter.import_external(id, textures)
    }
    #[cfg(feature = "gpu-interop")]
    pub fn drop_external_image(&mut self, id: ExternalImageId) {
        self.presenter.drop_external(id);
    }
}

/// Callbacks of a [`Shell`], confined to the thread running it.
pub trait Handler {
    /// Initialize retained state once the native window exists.
    fn start(
        &mut self,
        window: &mut Window<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError>;
    /// Handle one event and choose the next wake deadline.
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut Window<'_>,
    ) -> Result<ShellControl, ShellError>;
}

/// The main-thread window loop: one native window whose frames Arvane
/// draws on the GPU when it can (started with the window, on a worker
/// thread; the first frames are drawn on the CPU), else on the CPU.
/// Construct and run on the application's main thread.
pub struct Shell {
    shell: Box<dyn Backend>,
    prefer_gpu: bool,
    gpu_preference: GpuPreference,
    observer: Option<crate::presentation::Observer>,
}

impl Shell {
    /// Use the supplied window backend.
    pub fn new(backend: Box<dyn Backend>) -> Self {
        Self {
            shell: backend,
            prefer_gpu: true,
            gpu_preference: GpuPreference::default(),
            observer: None,
        }
    }

    /// Draw on the GPU when it is available (the default), falling back to
    /// the CPU; `false` draws on the CPU. The environment variable
    /// `ARVANE_RENDERER=cpu` overrides `true`, `ARVANE_RENDERER=gpu`
    /// overrides `false`.
    pub fn prefer_gpu(mut self, prefer: bool) -> Self {
        self.prefer_gpu = prefer;
        self
    }

    /// Make the window's GPU device as `preference` wishes (the adapter
    /// and the graphics API; the default is Arvane's own choice). The
    /// environment variables `WGPU_BACKEND` and `WGPU_POWER_PREF` override
    /// it. The device used is reported with the renderer's changes
    /// ([`RendererChange::Gpu`]).
    pub fn gpu_preference(mut self, preference: GpuPreference) -> Self {
        self.gpu_preference = preference;
        self
    }

    /// Call `observer` on the window's thread with every change of the
    /// renderer drawing the window's frames, as it happens, starting with
    /// the renderer the window starts with ([`RendererChange::Start`],
    /// when [`Shell::run`] starts).
    pub fn on_renderer_change(mut self, observer: impl FnMut(&RendererChange) + 'static) -> Self {
        self.observer = Some(Box::new(observer));
        self
    }

    /// Run until requested exit. A callback panic propagates out of `run`.
    pub fn run(self, handler: impl Handler) -> Result<(), ShellError> {
        let mut driver = driver(handler, self.prefer_gpu, self.gpu_preference, self.observer);
        self.shell.run(&mut driver)
    }
}

/// Presenter configuration shared by synchronous and stepped shells.
pub struct ShellSetup {
    pub prefer_gpu: bool,
    pub gpu_preference: GpuPreference,
    pub observer: Option<Observer>,
}
impl Default for ShellSetup {
    fn default() -> Self {
        Self {
            prefer_gpu: true,
            gpu_preference: GpuPreference::default(),
            observer: None,
        }
    }
}

/// A shell advanced by its caller.
pub struct SteppedShell(Box<dyn SteppedLoop>);
impl SteppedShell {
    pub fn step(&mut self, step: Step) -> StepOutcome {
        self.0.step(step)
    }
}
impl Shell {
    /// Start the same rendering driver on an explicitly stepped backend.
    pub fn start_stepped(
        backend: Box<dyn SteppedBackend>,
        setup: ShellSetup,
        handler: impl Handler + 'static,
    ) -> Result<SteppedShell, ShellError> {
        let driver = driver(
            handler,
            setup.prefer_gpu,
            setup.gpu_preference,
            setup.observer,
        );
        backend.start(Box::new(driver)).map(SteppedShell)
    }
}

/// Assemble the presenter and handler on the window's UI thread.
pub(crate) fn driver<H: Handler>(
    handler: H,
    prefer_gpu: bool,
    gpu_preference: GpuPreference,
    observer: Option<Observer>,
) -> Driver<H> {
    let env = crate::env::Env::read(gpu_preference);
    let gpu = crate::env::starts_gpu(env.renderer, prefer_gpu);
    let mut presenter = Presenter::new(Notes::new(observer), &env);
    presenter.note(RendererChange::Start { gpu });
    Driver {
        handler,
        presenter,
        gpu,
    }
}

/// The shell's handler: the host's [`Handler`] and the window's presenter.
pub(crate) struct Driver<H> {
    handler: H,
    presenter: Presenter,
    /// The window starts the GPU renderer.
    gpu: bool,
}

impl<H: Handler> ShellHandler for Driver<H> {
    fn close(&mut self, window: &mut dyn ShellWindow) {
        self.presenter.shutdown(window);
    }

    fn start(
        &mut self,
        window: &mut dyn ShellWindow,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        // On a worker thread: the first frames are drawn on the CPU.
        if self.gpu {
            self.presenter.start_gpu(waker.clone());
        }
        let mut window = Window {
            shell: window,
            presenter: &mut self.presenter,
        };
        self.handler.start(&mut window, waker)
    }

    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut dyn ShellWindow,
    ) -> Result<ShellControl, ShellError> {
        // A GPU start step finished: once the GPU can draw the window,
        // draw a frame now, while nothing else happens.
        if event == ShellEvent::Wake
            && window.surface_generation().is_some()
            && self.presenter.prepare(window)
        {
            window.request_redraw();
        }
        // The platform destroyed the window's surface: the GPU's goes with
        // it (the next frame after `SurfaceCreated` makes a new one).
        if event == ShellEvent::SurfaceDestroyed {
            self.presenter.surface_destroyed();
        }
        let mut window = Window {
            shell: window,
            presenter: &mut self.presenter,
        };
        self.handler.event(event, &mut window)
    }
}
