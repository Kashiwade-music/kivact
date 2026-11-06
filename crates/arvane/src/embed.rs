//! Role: Embedded window creation with the shared rendering driver.

use crate::presentation::Observer;
use crate::{Handler, RendererChange, ShellError};
pub use arvane_shell::PanicSlot;
use arvane_shell::{MakeHandler, ShellChild};

pub use arvane_shell::ChildHooks;
pub use kivact_types::EmbedOptions;

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

/// Build the rendering driver on the embedded window UI thread.
pub fn driver<H: Handler + 'static>(
    options: &EmbedOptions,
    make: impl FnOnce() -> Embedder<H> + Send + 'static,
) -> MakeHandler {
    let prefer_gpu = options.prefer_gpu;
    let gpu_preference = options.gpu_preference;
    Box::new(move || {
        let embedder = make();
        Box::new(crate::window::driver(
            embedder.handler,
            prefer_gpu,
            gpu_preference,
            embedder.observer,
        )) as Box<dyn arvane_shell::ShellHandler>
    })
}

/// A running child window. `Send` and `Sync`.
/// Drop closes it according to backend thread rules.
pub struct Embedded(Box<dyn ShellChild>);
impl Embedded {
    /// Own a running child window.
    pub fn new(child: Box<dyn ShellChild>) -> Self {
        Self(child)
    }
    /// Resize in physical pixels.
    pub fn set_size(&self, width: u32, height: u32) {
        self.0.set_size(width, height);
    }
    /// Last applied physical size.
    pub fn size(&self) -> (u32, u32) {
        self.0.size()
    }
    /// Set scale while keeping physical size.
    pub fn set_scale(&self, scale: f64) {
        self.0.set_scale(scale);
    }
    /// Last applied scale.
    pub fn scale(&self) -> f64 {
        self.0.scale()
    }
    /// Why the child stopped, or `None` while running.
    pub fn failure(&self) -> Option<ShellError> {
        self.0.failure()
    }
}
