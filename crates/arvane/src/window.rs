//! Role: Native window abstraction and lifecycle.
//!
//! Windows a host draws into: one trait ([`Window`]) for the native window
//! ([`NativeWindow`], run by [`Shell`]) and the test window
//! (`test_display::VirtualWindow`, feature `testing`), so a host is written
//! once for both. Each window presents through its own presenter
//! (`presenter.rs`): the host hands it a scene and learns what happened.
//!
//! The vocabulary of mobile platforms, touch and the application's life
//! cycle is here even where a platform does not use it, since adding it
//! later would break the trait. A window reports its environment
//! ([`Window::environment`]; changes as [`ShellEvent::EnvironmentChanged`],
//! the scale's as [`ShellEvent::Resized`]), what it can do
//! ([`Window::capabilities`]) and its surface's generation
//! ([`Window::surface_generation`]), and takes software keyboard requests
//! ([`Window::set_soft_keyboard`]). The application, the window and its
//! surface are apart: the application is suspended and resumed
//! ([`ShellEvent::Suspended`], [`ShellEvent::Resumed`]: in the
//! background), the surface destroyed and created again
//! ([`ShellEvent::SurfaceDestroyed`], [`ShellEvent::SurfaceCreated`]:
//! the presenter drops its GPU surface, also one being configured, and
//! presents nothing meanwhile), and the platform may run short of memory
//! ([`ShellEvent::MemoryWarning`]). Pointers have one vocabulary (id,
//! kind, phase, pressure, tilt): the mouse keeps its hover, button and
//! wheel events (id 1), touch and pen contacts arrive as
//! [`ShellEvent::Pointer`].
//!
//! Invariants:
//! - [`Window::defers_accessibility`] is a scheduling hint: it never
//!   changes what the tree holds.
//! - A test window fails its platform requests (copy, read, pick, save,
//!   pick a folder, open, reveal) through their `Result`s, the path a
//!   native window's failures take; the trait has no test-only way to
//!   inject or read failures.

use crate::presentation::{Notes, Observer};
use crate::presenter::Presenter;
use crate::{
    AccessibilityError, Failure, FileDialog, FramePlan, Presented, RendererChange, ShellControl,
    ShellError, ShellEvent, ShellImeText, ShellOptions, ShellWaker,
};
use arvane_shell::{ShellEventLoop, ShellHandler, ShellWindow};
use kivact_types::Environment;
use kivact_types::GpuPreference;
use kivact_types::render::{RenderScene, SemanticsTree, SemanticsUpdate};
use kivact_types::{Capabilities, Cursor, Rect, Size};
#[cfg(feature = "gpu-interop")]
use kivact_types::{ExternalImageId, GpuAdapter, SharedTextures};
use std::path::{Path, PathBuf};

/// A window a host draws into and takes its environment from.
pub trait Window {
    /// Request a logical content size.
    fn request_size(&mut self, size: Size) -> Result<(), ShellError>;
    /// Current logical content size.
    fn size(&self) -> Size;
    /// Physical pixels per logical pixel.
    fn scale(&self) -> f64;
    /// The window's clock (ns).
    fn now_ns(&self) -> u64;
    /// The window's environment now (scale, refresh period, preferences,
    /// safe area, mouse settings, locale). A change is announced by
    /// [`ShellEvent::EnvironmentChanged`], the scale's by
    /// [`ShellEvent::Resized`].
    fn environment(&self) -> Environment;
    /// What the backend the window runs on can do: the events it delivers
    /// and the requests it acts on (fixed for the window's life).
    fn capabilities(&self) -> Capabilities;
    /// The generation of the window's presentation surface (the count of
    /// surfaces the platform gave it, the first is 1), `None` while the
    /// platform destroyed it ([`ShellEvent::SurfaceDestroyed`] until
    /// [`ShellEvent::SurfaceCreated`]): nothing is presented then.
    fn surface_generation(&self) -> Option<u64>;
    /// Ask for a frame ([`ShellEvent::Redraw`]).
    fn request_redraw(&self);
    /// Release rebuildable renderer validation metadata under memory pressure.
    /// Live imports, leases and resources in use remain valid.
    fn release_caches(&mut self);
    /// Enter or leave borderless fullscreen.
    fn set_fullscreen(&self, fullscreen: bool) -> Result<(), ShellError>;
    /// Current fullscreen state.
    fn fullscreen(&self) -> bool;
    /// Change the title.
    fn set_title(&self, title: &str);
    /// Change the minimum logical content size.
    fn set_min_size(&self, size: Option<Size>);
    /// Enable the IME while an editable control has the focus.
    fn set_ime_allowed(&self, allowed: bool);
    /// Place the IME's candidate window at a logical caret rectangle.
    fn set_ime_cursor_area(&self, area: Rect);
    /// What the focused text input shows the IME (its queries:
    /// reconversion, the text around the composition, character boxes).
    fn set_ime_text(&self, text: Option<ShellImeText>);
    /// Show (an editable text input took the focus) or hide the software
    /// keyboard; the platform keeps the IME area
    /// ([`Window::set_ime_cursor_area`]) in view above it. A backend
    /// without one (desktops; [`kivact_types::Capabilities::soft_keyboard`])
    /// ignores the request.
    fn set_soft_keyboard(&self, shown: bool);
    /// Show `cursor` over the window's content.
    fn set_cursor(&self, cursor: Cursor);
    /// How the next frame is drawn: its logical extent and raster scale
    /// (the scale images are rasterized for).
    fn frame_plan(&self) -> FramePlan;
    /// Draw `scene` as `plan` says and show it. Arvane chooses the renderer
    /// and recovers its surface; the host follows [`Presented`] to decide
    /// when to request another frame. `Err` ends presentation when surface
    /// recovery fails or three surface failures occur in a row.
    fn present(&mut self, scene: &RenderScene, plan: FramePlan) -> Result<Presented, Failure>;
    /// Whether anything reads the accessibility tree yet (assistive
    /// technology asked for it; a test window is always read).
    fn accessibility_active(&self) -> bool;
    /// Whether the host should schedule accessibility after a pending
    /// frame, keeping tree construction off that frame's critical path.
    /// Native windows prefer this; test windows permit reading without
    /// frames. This is a scheduling hint: both tree operations validate
    /// and apply synchronously whenever the host calls them.
    fn defers_accessibility(&self) -> bool;
    /// Replace the accessibility tree with a complete one (validated; a
    /// rejected tree keeps the accepted one).
    fn replace_accessibility(&mut self, tree: &SemanticsTree) -> Result<(), AccessibilityError>;
    /// Apply the accessibility nodes that may have changed (validated; a
    /// rejected update keeps the accepted tree).
    fn update_accessibility(&mut self, update: &SemanticsUpdate) -> Result<(), AccessibilityError>;
    /// Write text to the clipboard.
    fn copy_text(&mut self, text: &str) -> Result<(), ShellError>;
    /// Read the clipboard's text.
    fn read_text(&mut self) -> Result<String, ShellError>;
    /// Ask for a file to open; `None` when cancelled.
    fn pick_file(&mut self, request: &FileDialog) -> Result<Option<PathBuf>, ShellError>;
    /// Ask for multiple files; `None` when cancelled.
    fn pick_files(&mut self, request: &FileDialog) -> Result<Option<Vec<PathBuf>>, ShellError>;
    /// Ask for a file to save; `None` when cancelled.
    fn save_file(&mut self, request: &FileDialog) -> Result<Option<PathBuf>, ShellError>;
    /// Ask for a folder; `None` when cancelled. The request's title and
    /// directory apply; its file name and filters do not.
    fn pick_folder(&mut self, request: &FileDialog) -> Result<Option<PathBuf>, ShellError>;
    /// Open the file or folder at `path` with the platform's default
    /// handler. `Ok` once the handler was started: what it does with the
    /// item afterwards is not reported. A platform without the service
    /// answers [`ShellError::unsupported`].
    fn open_path(&mut self, path: &Path) -> Result<(), ShellError>;
    /// Show the file or folder at `path` selected in the platform's file
    /// manager (the folder containing it where the file manager cannot
    /// select). `Ok` once the file manager was started; errors as for
    /// [`Window::open_path`].
    fn reveal_path(&mut self, path: &Path) -> Result<(), ShellError>;
    /// The adapter of the GPU device drawing the window's frames, when it
    /// can import textures an external render engine shares (feature
    /// `gpu-interop`); `None` while the window draws on the CPU, its device
    /// is not ready, or the adapter cannot import. The renderer's changes
    /// ([`crate::RendererChange`]) say when this changes; a new
    /// [`GpuAdapter::generation`] means every image imported before is
    /// gone.
    #[cfg(feature = "gpu-interop")]
    fn gpu_adapter(&self) -> Option<GpuAdapter>;
    /// Import the pool of textures an external render engine shares
    /// (`textures`, made as [`SharedTextures`] says) as the image `id` of
    /// this window, replacing the image `id` imported before; scenes then
    /// draw its frames with [`kivact_types::render::RenderCmd::External`]. The image
    /// belongs to this window's device: another window's scene that names
    /// `id` draws nothing. It lasts until [`Window::drop_external_image`],
    /// the window's close, or a new device (a new
    /// [`GpuAdapter::generation`]); then its release fence is signaled
    /// past every frame. Returns once the textures are ready for the
    /// engine to write.
    ///
    /// Errors: [`crate::FailureKind::Unavailable`] without a device that can
    /// import ([`Window::gpu_adapter`] is `None`), [`crate::FailureKind::Content`]
    /// when the textures do not fit what they say or the window's device
    /// (size, adapter, graphics API, handles it refuses),
    /// [`crate::FailureKind::Backend`] when the device failed.
    #[cfg(feature = "gpu-interop")]
    fn import_external_image(
        &mut self,
        id: ExternalImageId,
        textures: SharedTextures,
    ) -> Result<(), Failure>;
    /// Drop the image `id` (nothing when it is not imported): its frames
    /// draw nothing from now on, and its release fence is signaled past
    /// every frame of it.
    #[cfg(feature = "gpu-interop")]
    fn drop_external_image(&mut self, id: ExternalImageId);
}

/// The native window of a running [`Shell`], handed to the [`Handler`]'s
/// callbacks.
pub struct NativeWindow<'a> {
    shell: &'a mut dyn ShellWindow,
    presenter: &'a mut Presenter,
}

impl Window for NativeWindow<'_> {
    fn request_size(&mut self, size: Size) -> Result<(), ShellError> {
        self.shell.request_size(size)
    }
    fn size(&self) -> Size {
        self.shell.size()
    }
    fn scale(&self) -> f64 {
        self.shell.scale()
    }
    fn now_ns(&self) -> u64 {
        self.shell.now_ns()
    }
    fn environment(&self) -> Environment {
        self.shell.environment()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            gpu: cfg!(feature = "gpu"),
            ..self.shell.capabilities()
        }
    }
    fn surface_generation(&self) -> Option<u64> {
        self.shell.surface_generation()
    }
    fn request_redraw(&self) {
        self.shell.request_redraw();
    }
    fn set_fullscreen(&self, fullscreen: bool) -> Result<(), ShellError> {
        self.shell.set_fullscreen(fullscreen)
    }
    fn fullscreen(&self) -> bool {
        self.shell.fullscreen()
    }
    fn set_title(&self, title: &str) {
        self.shell.set_title(title);
    }
    fn set_min_size(&self, size: Option<Size>) {
        self.shell.set_min_size(size);
    }
    fn set_ime_allowed(&self, allowed: bool) {
        self.shell.set_ime_allowed(allowed);
    }
    fn set_ime_cursor_area(&self, area: Rect) {
        self.shell.set_ime_cursor_area(area);
    }
    fn set_ime_text(&self, text: Option<ShellImeText>) {
        self.shell.set_ime_text(text);
    }
    fn set_soft_keyboard(&self, shown: bool) {
        self.shell.set_soft_keyboard(shown);
    }
    fn set_cursor(&self, cursor: Cursor) {
        self.shell.set_cursor(cursor);
    }
    fn release_caches(&mut self) {
        self.presenter.release_caches();
    }
    fn frame_plan(&self) -> FramePlan {
        FramePlan::new(self.shell.physical_size(), self.shell.scale())
    }
    fn present(&mut self, scene: &RenderScene, plan: FramePlan) -> Result<Presented, Failure> {
        if self.shell.surface_generation().is_none() {
            return Ok(crate::presenter::no_surface());
        }
        if let Some(dropped) = crate::presenter::no_pixels(self.shell.physical_size()) {
            return Ok(dropped);
        }
        self.presenter.present_native(self.shell, scene, plan)
    }
    fn accessibility_active(&self) -> bool {
        self.shell.accessibility_active()
    }
    fn defers_accessibility(&self) -> bool {
        true
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
    fn pick_file(&mut self, request: &FileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.shell.pick_file(request)
    }
    fn pick_files(&mut self, request: &FileDialog) -> Result<Option<Vec<PathBuf>>, ShellError> {
        self.shell.pick_files(request)
    }
    fn save_file(&mut self, request: &FileDialog) -> Result<Option<PathBuf>, ShellError> {
        self.shell.save_file(request)
    }
    fn pick_folder(&mut self, request: &FileDialog) -> Result<Option<PathBuf>, ShellError> {
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
    ) -> Result<(), Failure> {
        self.presenter.import_external(id, textures)
    }
    #[cfg(feature = "gpu-interop")]
    fn drop_external_image(&mut self, id: ExternalImageId) {
        self.presenter.drop_external(id);
    }
}

/// Callbacks of a [`Shell`], confined to the thread running it.
pub trait Handler {
    /// Initialize retained state once the native window exists.
    fn start(
        &mut self,
        window: &mut NativeWindow<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError>;
    /// Handle one event and choose the next wake deadline.
    fn event(
        &mut self,
        event: ShellEvent,
        window: &mut NativeWindow<'_>,
    ) -> Result<ShellControl, ShellError>;
}

/// The main-thread window loop: one native window whose frames Arvane
/// draws on the GPU when it can (started with the window, on a worker
/// thread; the first frames are drawn on the CPU), else on the CPU.
/// Construct and run on the application's main thread.
pub struct Shell {
    shell: Box<dyn ShellEventLoop>,
    prefer_gpu: bool,
    gpu_preference: GpuPreference,
    observer: Option<crate::presentation::Observer>,
}

impl Shell {
    /// Create the native event loop, rejecting invalid geometry and OS
    /// failures.
    pub fn new(options: ShellOptions) -> Result<Self, ShellError> {
        Ok(Self {
            shell: Box::new(arvane_native::Shell::new(options)?),
            prefer_gpu: true,
            gpu_preference: GpuPreference::default(),
            observer: None,
        })
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
        let mut driver = driver(
            handler,
            self.prefer_gpu,
            self.gpu_preference,
            self.observer,
            false,
        );
        self.shell.run(&mut driver)
    }
}

/// Assemble the presenter and handler on the window's UI thread.
pub(crate) fn driver<H: Handler>(
    handler: H,
    prefer_gpu: bool,
    gpu_preference: GpuPreference,
    observer: Option<Observer>,
    embedded: bool,
) -> Driver<H> {
    let env = crate::env::Env::read(gpu_preference);
    let gpu = crate::env::starts_gpu(env.renderer, prefer_gpu);
    let mut presenter = Presenter::new(Notes::new(observer), &env);
    presenter.note(RendererChange::Start { gpu });
    Driver {
        handler,
        presenter,
        gpu,
        _retention: crate::shutdown::RetainOnDrop(embedded),
    }
}

/// The shell's handler: the host's [`Handler`] and the window's presenter.
pub(crate) struct Driver<H> {
    handler: H,
    presenter: Presenter,
    /// The window starts the GPU renderer.
    gpu: bool,
    /// Dropped last (fields drop in declaration order): it retains the module only after the handler, its asset stores and the presenter dropped (`shutdown.rs`).
    _retention: crate::shutdown::RetainOnDrop,
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
        let mut window = NativeWindow {
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
        let mut window = NativeWindow {
            shell: window,
            presenter: &mut self.presenter,
        };
        self.handler.event(event, &mut window)
    }
}
