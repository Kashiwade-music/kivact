//! Role: Child window options and the host event loop SPI.

use crate::{ShellError, ShellHandler};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// The panic of an embedded child's App, kept where it reached a platform
/// boundary until the embedder's Rust code resumes it on its own thread.
///
/// Clones share the slot. It keeps the first payload; a later one is dropped
/// (the panic hook already printed it).
#[derive(Clone, Default)]
pub struct PanicSlot(Arc<Mutex<Option<Box<dyn Any + Send>>>>);

impl PanicSlot {
    /// Keep `payload` unless a payload is already kept.
    pub fn put(&self, payload: Box<dyn Any + Send>) {
        let mut slot = self.lock();
        if slot.is_none() {
            *slot = Some(payload);
        }
    }

    /// Take the kept payload, if any.
    pub fn take(&self) -> Option<Box<dyn Any + Send>> {
        self.lock().take()
    }

    /// Whether a payload is kept.
    pub fn is_set(&self) -> bool {
        self.lock().is_some()
    }

    /// Resume the kept panic, if any, on the calling thread with its
    /// original payload.
    pub fn resume(&self) {
        if let Some(payload) = self.take() {
            std::panic::resume_unwind(payload);
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Box<dyn Any + Send>>> {
        self.0
            .lock()
            .expect("panic slot lock poisoned: a panic already happened")
    }
}

impl std::fmt::Debug for PanicSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PanicSlot").field(&self.is_set()).finish()
    }
}

/// The first logical size and optional host-decided scale of a child window.
/// The host's window is passed beside the options, not inside them.
///
/// The default is 960 by 720 logical pixels, the OS scale and no resize
/// request.
#[derive(Clone)]
pub struct ChildOptions {
    /// Initial width in logical pixels (default 960).
    pub width: f32,
    /// Initial height in logical pixels (default 720).
    pub height: f32,
    /// Physical pixels per logical pixel, or the OS scale.
    pub scale: Option<f32>,
    /// Ask the host for a physical content size.
    pub on_resize_request: Option<std::sync::Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    /// Where a panic of the child's App is kept for the embedder to resume.
    pub panic: PanicSlot,
}

impl std::fmt::Debug for ChildOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildOptions")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale", &self.scale)
            .field("on_resize_request", &self.on_resize_request.is_some())
            .field("panic", &self.panic.is_set())
            .finish()
    }
}

impl Default for ChildOptions {
    fn default() -> Self {
        Self {
            width: 960.0,
            height: 720.0,
            scale: None,
            on_resize_request: None,
            panic: PanicSlot::default(),
        }
    }
}

/// Builds the window's handler on its UI thread, once.
pub type MakeHandler = Box<dyn FnOnce() -> Box<dyn ShellHandler> + Send>;

/// A child in a host event loop. Drop closes it using the backend's thread rules.
pub trait ShellChild: Send + Sync {
    /// Resize in physical pixels.
    fn set_size(&self, width: u32, height: u32);
    /// Last applied physical size.
    fn size(&self) -> (u32, u32);
    /// Change scale, keeping the physical size.
    fn set_scale(&self, scale: f32);
    /// Last applied scale.
    fn scale(&self) -> f32;
    /// Why the window stopped, or `None` while running.
    fn failure(&self) -> Option<ShellError>;
}
