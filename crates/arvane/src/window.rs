//! Role: One concrete rendering window and shared backend lifecycle driver.

use crate::presentation::{Notes, Observer};
use crate::presenter::{ExportNotice, Presenter};
use crate::{FramePlan, Presented};
use arvane_protocol::{Composition, SemanticsTree, SemanticsUpdate};
use arvane_shell::{
    AccessibilityError, Backend, EventTarget, InjectedInput, MakeHandler, ShellControl, ShellError,
    ShellEvent, ShellHandler, ShellLoop, ShellWaker, ShellWindow, Step, StepOutcome,
    SteppedBackend, SteppedLoop, WakeReasons,
};
use kivact_types::Environment;
use kivact_types::Failure;
use kivact_types::FailureKind;
use kivact_types::GpuPreference;
use kivact_types::ids::DrawableId;
use kivact_types::{Capabilities, Cursor, RequestId, Size, WindowId, WindowRequest};
use kivact_types::{ExternalImageId, GpuAdapter, SharedTextures, WindowContent};
use std::collections::BTreeMap;

/// The native window of a running [`run`] loop, handed to the [`Handler`]'s
/// callbacks. A foreign-content window has no presenter.
pub struct Window<'a> {
    shell: &'a mut dyn ShellWindow,
    presenter: Option<&'a mut Presenter>,
}

/// The error of a renderer call on a foreign-content window, which has none.
fn no_renderer() -> Failure {
    Failure::new(
        FailureKind::Unavailable,
        "a foreign-content window has no renderer",
    )
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
    /// Whether the window can show a frame now, and on which surface generation.
    pub fn drawable(&self) -> arvane_shell::Drawable {
        self.shell.drawable()
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
    /// Enable IME input for the focused editor as `ime` asks, or disable it.
    pub fn set_ime(&self, ime: Option<arvane_shell::ImeRequest>) {
        self.shell.set_ime(ime);
    }
    /// The native child window of a foreign-content window; `None` otherwise.
    pub fn foreign_child(&self) -> Option<kivact_types::NativeWindow> {
        self.shell.foreign_child()
    }
    pub fn set_cursor(&self, cursor: Cursor) {
        self.shell.set_cursor(cursor);
    }
    pub fn release_caches(&mut self) {
        if let Some(presenter) = self.presenter.as_deref_mut() {
            presenter.release_caches();
        }
    }
    /// Release the window's CPU frame state after it has been idle. The next
    /// frame redraws everything; what the window shows stays on screen.
    pub fn forget_frame_state(&mut self) {
        if let Some(presenter) = self.presenter.as_deref_mut() {
            presenter.forget_frame_state();
        }
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
        let drawable = self.shell.drawable();
        if !matches!(drawable, arvane_shell::Drawable::Yes { .. }) {
            return Ok(Presented::NotDrawable(drawable));
        }
        let Some(presenter) = self.presenter.as_deref_mut() else {
            return Ok(Presented::NotDrawable(arvane_shell::Drawable::NoSurface));
        };
        presenter.present_native(self.shell, composition, plan)
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
        self.presenter
            .as_deref()
            .and_then(|presenter| presenter.gpu_adapter())
    }
    pub fn import_external_image(
        &mut self,
        id: ExternalImageId,
        textures: SharedTextures,
    ) -> Result<(), Failure> {
        match self.presenter.as_deref_mut() {
            Some(presenter) => presenter.import_external(id, textures),
            None => Err(no_renderer()),
        }
    }
    pub fn drop_external_image(&mut self, id: ExternalImageId) {
        if let Some(presenter) = self.presenter.as_deref_mut() {
            presenter.drop_external(id);
        }
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
        match self.presenter.as_deref_mut() {
            Some(presenter) => presenter.export_drawable(sink, generation, adapter, size),
            None => Err(no_renderer()),
        }
    }

    /// Drop the GPU export pool of drawable `sink`.
    pub fn drop_export(&mut self, sink: DrawableId) {
        if let Some(presenter) = self.presenter.as_deref_mut() {
            presenter.drop_export(sink);
        }
    }

    /// Pool losses and GPU availability since the last call.
    pub fn take_export_notices(&mut self) -> Vec<ExportNotice> {
        match self.presenter.as_deref_mut() {
            Some(presenter) => presenter.take_export_notices(),
            None => Vec::new(),
        }
    }

    /// Make the window's GPU device lost, as a device failure does (test control).
    #[cfg(feature = "test")]
    pub(crate) fn lose_gpu_device(&mut self) {
        if let Some(presenter) = self.presenter.as_deref_mut() {
            presenter.lose_gpu_device();
        }
    }
}

/// Callbacks of a [`run`] loop, confined to the thread running it.
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
    /// The renderer observer, cloned into the notes of every presenter.
    observer: &'a Option<Observer>,
}

impl<'a> Windows<'a> {
    /// The window `id` with its presenter, or `None` once it is closed.
    pub fn get(&mut self, id: WindowId) -> Option<Window<'_>> {
        let presenter = self.presenters.get_mut(&id);
        let shell = self.shell.window(id)?;
        Some(Window { shell, presenter })
    }

    /// The open windows, ascending by id.
    pub fn ids(&self) -> Vec<WindowId> {
        self.shell.windows()
    }

    /// Make a presenter for a drawn window (and start its GPU when preferred),
    /// then open the shell window. A foreign-content window has none.
    pub fn open(&mut self, request: WindowRequest) -> Result<(), ShellError> {
        let id = request.id;
        if request.content == WindowContent::Drawn {
            let presenter = Presenter::new(Notes::new(self.observer.clone(), id), &self.env);
            self.presenters.insert(id, presenter);
        }
        if let Err(error) = self.shell.open_window(request) {
            self.presenters.remove(&id);
            return Err(error);
        }
        // Only an opened window starts its renderer.
        let period = self
            .shell
            .window(id)
            .map(|window| window.environment().frame_period_ns);
        if let Some(presenter) = self.presenters.get_mut(&id) {
            if let Some(period) = period {
                presenter.set_frame_period(period);
            }
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

/// How the driver draws: when the GPU device starts and on which adapter,
/// and who is told of renderer changes.
#[derive(Default)]
pub struct DriverOptions {
    /// The GPU device's adapter, API and start (`GpuStart::Never`: CPU only).
    /// `ARVANE_RENDERER`, `WGPU_BACKEND` and `WGPU_POWER_PREF` override it.
    pub gpu: GpuPreference,
    /// Called on the window's thread with every renderer change, starting
    /// with the main window's `RendererChange::Start`.
    pub observer: Option<Observer>,
}

/// Run `backend`'s window loop with the driver until it exits. Construct
/// and run on the application's main thread. A callback panic propagates.
pub fn run(
    backend: Box<dyn Backend>,
    options: DriverOptions,
    handler: impl Handler,
) -> Result<(), ShellError> {
    let mut driver = driver(handler, options);
    backend.run(&mut driver)
}

/// Start the same driver on an explicitly stepped backend.
pub fn start_stepped(
    backend: Box<dyn SteppedBackend>,
    options: DriverOptions,
    handler: impl Handler + 'static,
) -> Result<SteppedShell, ShellError> {
    backend
        .start(Box::new(driver(handler, options)))
        .map(SteppedShell)
}

/// A shell advanced by its caller.
pub struct SteppedShell(Box<dyn SteppedLoop>);
impl SteppedShell {
    pub fn step(&mut self, step: Step) -> StepOutcome {
        self.0.step(step)
    }
}

/// Assemble the handler and the renderers on the window's UI thread.
pub(crate) fn driver<H: Handler>(handler: H, options: DriverOptions) -> Driver<H> {
    let env = crate::env::Env::read(options.gpu);
    Driver {
        handler,
        presenters: BTreeMap::new(),
        env,
        observer: options.observer,
    }
}

/// The shell's handler: the host's [`Handler`] and every window's presenter.
pub(crate) struct Driver<H> {
    handler: H,
    presenters: BTreeMap<WindowId, Presenter>,
    /// The renderers' environment, read once per loop.
    env: crate::env::Env,
    /// The renderer observer, cloned into the notes of every presenter.
    observer: Option<Observer>,
}

impl<H: Handler> ShellHandler for Driver<H> {
    fn close(&mut self, shell: &mut dyn ShellLoop) {
        let mut windows = Windows {
            shell,
            presenters: &mut self.presenters,
            env: self.env,
            observer: &self.observer,
        };
        self.handler.close(&mut windows);
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
        if let Some(window) = shell.window(WindowId::MAIN) {
            presenter.set_frame_period(window.environment().frame_period_ns);
        }
        presenter.begin(self.env.start, waker.clone());
        self.presenters.insert(WindowId::MAIN, presenter);
        let mut windows = Windows {
            shell,
            presenters: &mut self.presenters,
            env: self.env,
            observer: &self.observer,
        };
        self.handler.start(&mut windows, waker)
    }

    fn event(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        shell: &mut dyn ShellLoop,
    ) -> Result<ShellControl, ShellError> {
        // The display period is read again when the window's environment may
        // have changed (a move to another monitor, a resize, the focus).
        if let EventTarget::Window(id) = target
            && matches!(
                event,
                ShellEvent::EnvironmentChanged | ShellEvent::Resized | ShellEvent::Focused(true)
            )
            && let Some(window) = shell.window(id)
            && let Some(presenter) = self.presenters.get_mut(&id)
        {
            presenter.set_frame_period(window.environment().frame_period_ns);
        }
        // A GPU start step finished: once the GPU can draw a window, draw a
        // frame now, while nothing else happens.
        if matches!(event, ShellEvent::Wake(reasons) if reasons.contains(WakeReasons::GPU_STEP)) {
            for (id, presenter) in &mut self.presenters {
                let Some(window) = shell.window(*id) else {
                    continue;
                };
                if matches!(window.drawable(), arvane_shell::Drawable::Yes { .. })
                    && presenter.prepare(window)
                {
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
            observer: &self.observer,
        };
        self.handler.event(target, event, &mut windows)
    }
}

/// The driver of an embedded window, made on the window's UI thread:
/// `make` builds the handler and the optional renderer observer there.
pub fn embed_driver<H: Handler + 'static>(
    gpu: GpuPreference,
    make: impl FnOnce() -> (H, Option<Observer>) + Send + 'static,
) -> MakeHandler {
    Box::new(move || {
        let (handler, observer) = make();
        Box::new(driver(handler, DriverOptions { gpu, observer })) as Box<dyn ShellHandler>
    })
}
