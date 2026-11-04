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

/// Callback and panic state owned by the shell, separate from pure options.
#[derive(Clone, Default)]
pub struct ChildHooks {
    /// Ask the embedding host for a physical content size.
    pub on_resize_request: Option<Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    /// The existing shared panic payload slot.
    pub panic: PanicSlot,
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
    fn set_scale(&self, scale: f64);
    /// Last applied scale.
    fn scale(&self) -> f64;
    /// Why the window stopped, or `None` while running.
    fn failure(&self) -> Option<ShellError>;
}
