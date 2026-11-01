//! Role: Embedded window creation with the shared rendering driver.

use crate::presentation::Observer;
use crate::{Handler, RendererChange, ShellError};
pub use arvane_shell::PanicSlot;
use arvane_shell::{ChildOptions, ShellChild};
use kivact_types::{GpuPreference, ParentWindow, Size};

/// Initial embedded window settings. Coordinates are logical pixels. The
/// host's window is passed to [`open`] beside the options.
///
/// The default is 960 by 720, the OS scale, no resize request, GPU
/// rendering preferred and the default GPU preference.
pub struct EmbedOptions {
    /// Initial logical size (default 960 by 720).
    pub size: Size,
    /// Host-decided scale, or the OS scale.
    pub scale: Option<f32>,
    /// Ask the host for a physical content size.
    pub on_resize_request: Option<std::sync::Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    /// Prefer GPU rendering, as `Shell::prefer_gpu` does (default true).
    pub prefer_gpu: bool,
    /// GPU device preference.
    pub gpu_preference: GpuPreference,
    /// Where a panic of the child's App is kept for the embedder to resume
    /// (see [`PanicSlot`]).
    pub panic: PanicSlot,
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            size: Size {
                width: 960.0,
                height: 720.0,
            },
            scale: None,
            on_resize_request: None,
            prefer_gpu: true,
            gpu_preference: GpuPreference::default(),
            panic: PanicSlot::default(),
        }
    }
}

/// Handler and optional renderer observer constructed on the UI thread.
pub struct Embedder<H> {
    handler: H,
    observer: Option<Observer>,
}

impl<H: Handler> Embedder<H> {
    /// Use this handler for the embedded window.
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            observer: None,
        }
    }
    /// Observe renderer changes on the window's UI thread.
    pub fn on_renderer_change(mut self, observer: impl FnMut(&RendererChange) + 'static) -> Self {
        self.observer = Some(Box::new(observer));
        self
    }
}

/// Open a child of the host window `parent`, building its handler once on the
/// UI thread.
pub fn open<H, F>(
    parent: ParentWindow,
    options: EmbedOptions,
    make: F,
) -> Result<Embedded, ShellError>
where
    H: Handler + 'static,
    F: FnOnce() -> Embedder<H> + Send + 'static,
{
    let child = ChildOptions {
        width: options.size.width,
        height: options.size.height,
        scale: options.scale,
        on_resize_request: options.on_resize_request,
        panic: options.panic,
    };
    let make = Box::new(move || {
        let embedder = make();
        Box::new(crate::window::driver(
            embedder.handler,
            options.prefer_gpu,
            options.gpu_preference,
            embedder.observer,
            true,
        )) as Box<dyn arvane_shell::ShellHandler>
    });
    arvane_native::embed::open(parent, child, make).map(Embedded)
}

/// A running child window. `Send` and `Sync`.
/// Drop closes it according to backend thread rules.
pub struct Embedded(Box<dyn ShellChild>);
impl Embedded {
    /// Resize in physical pixels.
    pub fn set_size(&self, width: u32, height: u32) {
        self.0.set_size(width, height);
    }
    /// Last applied physical size.
    pub fn size(&self) -> (u32, u32) {
        self.0.size()
    }
    /// Set scale while keeping physical size.
    pub fn set_scale(&self, scale: f32) {
        self.0.set_scale(scale);
    }
    /// Last applied scale.
    pub fn scale(&self) -> f32 {
        self.0.scale()
    }
    /// Why the child stopped, or `None` while running.
    pub fn failure(&self) -> Option<ShellError> {
        self.0.failure()
    }
}
