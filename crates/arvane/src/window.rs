//! Role: One concrete rendering window and shared backend lifecycle driver.

use crate::presentation::{Notes, Observer};
use crate::presenter::{ExportNotice, Presenter};
use crate::{
    AccessibilityError, Failure, FramePlan, Presented, RendererChange, ShellControl, ShellError,
    ShellEvent, ShellImeText, ShellWaker,
};
use arvane_shell::{
    Backend, EventTarget, InjectedInput, ShellHandler, ShellLoop, ShellWindow, Step, StepOutcome,
    SteppedBackend, SteppedLoop, WakeReasons,
};
use kivact_types::Environment;
use kivact_types::GpuPreference;
use kivact_types::ids::DrawableId;
use kivact_types::render::{Composition, SemanticsTree, SemanticsUpdate};
use kivact_types::{Capabilities, Cursor, Rect, RequestId, Size, WindowId, WindowRequest};
use kivact_types::{ExternalImageId, GpuAdapter, SharedTextures};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

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
    /// Current physical presentation width and height in pixels.
    pub fn physical_size(&self) -> (u32, u32) {
        self.shell.physical_size()
    }
    pub fn request_size(
        &mut self,
        size: Size,
        request: kivact_types::RequestId,
    ) -> arvane_shell::SizeReply {
        self.shell.request_size(size, request)
    }
    pub fn unhandled_key(&mut self, code: kivact_types::KeyCode) {
        self.shell.unhandled_key(code);
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
            gpu: arvane_compose::GPU_BUILT,
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
    pub fn set_resizable(&self, resizable: bool) {
        self.shell.set_resizable(resizable);
    }
    pub fn set_ime_purpose(&self, purpose: kivact_types::InputPurpose) {
        self.shell.set_ime_purpose(purpose);
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
    /// Release the window's CPU frame state after it has been idle. The next
    /// frame redraws everything; what the window shows stays on screen.
    pub fn forget_frame_state(&mut self) {
        self.presenter.forget_frame_state();
    }
    pub fn frame_plan(&self) -> FramePlan {
        FramePlan::new(self.shell.physical_size(), self.shell.scale())
    }
    /// Present one frame: `composition` drawn at `plan`'s size and scale.
    pub fn present(
        &mut self,
        composition: &Composition,
        plan: FramePlan,
    ) -> Result<Presented, Failure> {
        if self.shell.surface_generation().is_none() {
            return Ok(crate::presenter::no_surface());
        }
        if let Some(dropped) = crate::presenter::no_pixels(self.shell.physical_size()) {
            return Ok(dropped);
        }
        self.presenter.present_native(self.shell, composition, plan)
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

    pub fn gpu_adapter(&self) -> Option<GpuAdapter> {
        self.presenter.gpu_adapter()
    }
    pub fn import_external_image(
        &mut self,
        id: ExternalImageId,
        textures: SharedTextures,
    ) -> Result<(), Failure> {
        self.presenter.import_external(id, textures)
    }
    pub fn drop_external_image(&mut self, id: ExternalImageId) {
        self.presenter.drop_external(id);
    }

    /// Make the GPU export pool of drawable `sink` for request `generation`
    /// on `adapter`, sized `size` px. A failure is kind `NotReady` while the
    /// device is starting (retry when it reports ready), and `Unavailable` in
    /// a build without GPU support.
    pub fn export_drawable(
        &mut self,
        sink: DrawableId,
        generation: u64,
        adapter: &GpuAdapter,
        size: (u32, u32),
    ) -> Result<SharedTextures, Failure> {
        self.presenter
            .export_drawable(sink, generation, adapter, size)
    }

    /// Drop the GPU export pool of drawable `sink`.
    pub fn drop_export(&mut self, sink: DrawableId) {
        self.presenter.drop_export(sink);
    }

    /// Pool losses and GPU availability since the last call.
    pub fn take_export_notices(&mut self) -> Vec<ExportNotice> {
        self.presenter.take_export_notices()
    }

    /// Make the window's GPU device lost, as a device failure does (test control).
    pub(crate) fn lose_gpu_device(&mut self) {
        self.presenter.lose_gpu_device();
    }
}

/// Callbacks of a [`Shell`], confined to the thread running it.
pub trait Handler {
    /// Initialize retained state once the main window exists.
    fn start(
        &mut self,
        windows: &mut Windows<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError>;
    /// Handle one event and choose the next wake deadline.
    fn event(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError>;
    /// Release what the handler holds; the windows are still open.
    fn close(&mut self, windows: &mut Windows<'_>);
}

/// The open windows with their presenters, during one callback.
pub struct Windows<'a> {
    shell: &'a mut dyn ShellLoop,
    presenters: &'a mut BTreeMap<WindowId, Presenter>,
    env: crate::env::Env,
    // xtask-allow(state-fields): option: the renderer observer the application gave (`Shell::on_renderer_change` or `ShellSetup::observer`), cloned into every presenter's Notes; absent when it gave none
    /// The renderer observer, cloned into the notes of every presenter.
    observer: Option<Observer>,
}

impl<'a> Windows<'a> {
    /// The window `id` with its presenter, or `None` once it is closed.
    pub fn get(&mut self, id: WindowId) -> Option<Window<'_>> {
        let presenter = self.presenters.get_mut(&id)?;
        let shell = self.shell.window(id)?;
        Some(Window { shell, presenter })
    }

    /// The open windows, ascending by id.
    pub fn ids(&self) -> Vec<WindowId> {
        self.presenters.keys().copied().collect()
    }

    /// Make a presenter (and start its GPU when preferred), then open the
    /// shell window.
    pub fn open(&mut self, request: WindowRequest) -> Result<(), ShellError> {
        let id = request.id;
        let presenter = Presenter::new(Notes::new(self.observer.clone(), id), &self.env);
        self.presenters.insert(id, presenter);
        if let Err(error) = self.shell.open_window(request) {
            self.presenters.remove(&id);
            return Err(error);
        }
        // Only an opened window starts its renderer.
        if let Some(presenter) = self.presenters.get_mut(&id) {
            presenter.begin(self.env.start, self.shell.waker());
        }
        Ok(())
    }

    /// Start `id`'s GPU device when it waits for a reason (`GpuStart::OnDemand`):
    /// the App reads the window's adapter to share GPU textures with it. Nothing
    /// for a window on the CPU only or whose device started.
    pub fn demand_gpu(&mut self, id: WindowId) {
        if let Some(presenter) = self.presenters.get_mut(&id) {
            presenter.demand_gpu();
        }
    }

    /// Shut the presenter down, then close the shell window.
    pub fn close(&mut self, id: WindowId) {
        if let Some(window) = self.shell.window(id)
            && let Some(presenter) = self.presenters.get_mut(&id)
        {
            presenter.shutdown(window);
        }
        self.presenters.remove(&id);
        self.shell.close_window(id);
    }

    /// Queue input for `id`; see [`ShellLoop::inject`].
    pub fn inject(&mut self, id: WindowId, inputs: Vec<InjectedInput>, request: RequestId) {
        self.shell.inject(id, inputs, request);
    }

    /// The loop clock, comparable to [`ShellControl::WakeAt`].
    pub fn now_ns(&self) -> u64 {
        self.shell.now_ns()
    }

    /// What the loop can do (`multi_window`, `window_placement`, ...), with
    /// the GPU that Arvane adds to every window.
    pub fn capabilities(&self) -> Capabilities {
        Capabilities {
            gpu: arvane_compose::GPU_BUILT,
            ..self.shell.capabilities()
        }
    }
}

/// The main-thread window loop: one native window whose frames Arvane
/// draws on the GPU when it can (started with the window or, by default,
/// when the window needs it, on a worker thread; the first frames are
/// drawn on the CPU), else on the CPU. Construct and run on the
/// application's main thread.
pub struct Shell {
    shell: Box<dyn Backend>,
    // xtask-allow(state-fields): option: whether the application asked to draw on the GPU (`Shell::prefer_gpu`, default true); false draws on the CPU unless `ARVANE_RENDERER` is `gpu` or `auto`
    prefer_gpu: bool,
    gpu_preference: GpuPreference,
    // xtask-allow(state-fields): option: the renderer observer `Shell::on_renderer_change` gave; absent when the application observes no renderer change
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
    /// overrides `false` (the device starts with the window), and
    /// `ARVANE_RENDERER=auto` starts the device when it is needed.
    pub fn prefer_gpu(mut self, prefer: bool) -> Self {
        self.prefer_gpu = prefer;
        self
    }

    /// Make the window's GPU device as `preference` wishes (the adapter,
    /// the graphics API and when the device is started; the default is
    /// Arvane's own choice, started when the window needs it). The
    /// environment variables `WGPU_BACKEND` and `WGPU_POWER_PREF` override
    /// the first two. The device used is reported with the renderer's
    /// changes ([`RendererChange::Gpu`]).
    pub fn gpu_preference(mut self, preference: GpuPreference) -> Self {
        self.gpu_preference = preference;
        self
    }

    /// Call `observer` on the window's thread with every change of the
    /// renderer drawing a window's frames, as it happens, starting with the
    /// renderer the main window starts with ([`RendererChange::Start`],
    /// when [`Shell::run`] starts).
    pub fn on_renderer_change(
        mut self,
        observer: impl FnMut(WindowId, &RendererChange) + 'static,
    ) -> Self {
        self.observer = Some(Rc::new(RefCell::new(observer)));
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
    // xtask-allow(state-fields): option: whether the host asks to draw on the GPU (default true); false draws on the CPU unless `ARVANE_RENDERER` is `gpu` or `auto`
    pub prefer_gpu: bool,
    pub gpu_preference: GpuPreference,
    // xtask-allow(state-fields): option: the renderer observer the host gives, cloned into every presenter's Notes; absent when it observes no renderer change
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

/// Assemble the handler and the renderers on the window's UI thread.
pub(crate) fn driver<H: Handler>(
    handler: H,
    prefer_gpu: bool,
    gpu_preference: GpuPreference,
    observer: Option<Observer>,
) -> Driver<H> {
    let env = crate::env::Env::read(prefer_gpu, gpu_preference);
    Driver {
        handler,
        presenters: BTreeMap::new(),
        env,
        observer,
    }
}

/// The shell's handler: the host's [`Handler`] and every window's presenter.
pub(crate) struct Driver<H> {
    handler: H,
    presenters: BTreeMap<WindowId, Presenter>,
    /// The renderers' environment, read once per loop.
    env: crate::env::Env,
    // xtask-allow(state-fields): option: the renderer observer the application gave (`Shell::on_renderer_change` or `ShellSetup::observer`), cloned into every presenter's Notes; absent when it gave none
    /// The renderer observer, cloned into the notes of every presenter.
    observer: Option<Observer>,
}

impl<H: Handler> ShellHandler for Driver<H> {
    fn close(&mut self, shell: &mut dyn ShellLoop) {
        let mut windows = Windows {
            shell,
            presenters: &mut self.presenters,
            env: self.env,
            observer: self.observer.clone(),
        };
        self.handler.close(&mut windows);
        drop(windows);
        for (id, mut presenter) in std::mem::take(&mut self.presenters) {
            if let Some(window) = shell.window(id) {
                presenter.shutdown(window);
            }
        }
    }

    fn start(
        &mut self,
        shell: &mut dyn ShellLoop,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        // On a worker thread: the first frames are drawn on the CPU.
        let mut presenter =
            Presenter::new(Notes::new(self.observer.clone(), WindowId::MAIN), &self.env);
        presenter.begin(self.env.start, waker.clone());
        self.presenters.insert(WindowId::MAIN, presenter);
        let mut windows = Windows {
            shell,
            presenters: &mut self.presenters,
            env: self.env,
            observer: self.observer.clone(),
        };
        self.handler.start(&mut windows, waker)
    }

    fn event(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        shell: &mut dyn ShellLoop,
    ) -> Result<ShellControl, ShellError> {
        // A GPU start step finished: once the GPU can draw a window, draw a
        // frame now, while nothing else happens.
        if matches!(event, ShellEvent::Wake(reasons) if reasons.contains(WakeReasons::GPU_STEP)) {
            for (id, presenter) in &mut self.presenters {
                let Some(window) = shell.window(*id) else {
                    continue;
                };
                if window.surface_generation().is_some() && presenter.prepare(window) {
                    window.request_redraw();
                }
            }
        }
        // The platform destroyed a window's surface: the GPU's goes with it
        // (the next frame after `SurfaceCreated` makes a new one).
        if event == ShellEvent::SurfaceDestroyed
            && let EventTarget::Window(id) = target
            && let Some(presenter) = self.presenters.get_mut(&id)
        {
            presenter.surface_destroyed();
        }
        let mut windows = Windows {
            shell,
            presenters: &mut self.presenters,
            env: self.env,
            observer: self.observer.clone(),
        };
        self.handler.event(target, event, &mut windows)
    }
}
