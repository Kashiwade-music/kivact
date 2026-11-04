//! Role: Embedded hosts using the desktop client's turn and rendering loop.

use crate::api::Client;
use crate::contract::HostError;
use crate::{FrameLog, host};
pub use arvane::embed::ChildHooks;
pub use arvane::embed::PanicSlot;
use arvane::embed::{Embedded, Embedder};
pub use arvane_native::retain_module;
use kivact_types::EmbedOptions;
use kivact_types::ParentWindow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// An embedded engine host. `Send` and `Sync`. Drop requests closure and stops new turns.
pub struct EmbeddedHost {
    host: Embedded,
    closing: Arc<AtomicBool>,
}
impl Drop for EmbeddedHost {
    fn drop(&mut self) {
        // The UI may still be inside Host::effects. Stop its next client turn
        // before the lower native handle can defer destruction of this host.
        self.closing.store(true, Ordering::Release);
    }
}
impl EmbeddedHost {
    /// Resize in physical pixels.
    pub fn set_size(&self, width: u32, height: u32) {
        self.host.set_size(width, height);
    }
    /// Last applied physical size.
    pub fn size(&self) -> (u32, u32) {
        self.host.size()
    }
    /// Set scale while preserving physical size.
    pub fn set_scale(&self, scale: f64) {
        self.host.set_scale(scale);
    }
    /// Last applied scale.
    pub fn scale(&self) -> f64 {
        self.host.scale()
    }
    /// Why the host stopped, or `None` while running.
    pub fn failure(&self) -> Option<HostError> {
        self.host.failure().map(|e| HostError::new(e.to_string()))
    }
}

/// Start a client inside a host window; build the client on its UI thread
/// with the close flag the returned host sets when it is dropped (the client
/// stops delivering the rest of its event batch once it is set).
pub fn embed_native(
    parent: ParentWindow,
    options: EmbedOptions,
    hooks: ChildHooks,
    client: impl FnOnce(Arc<AtomicBool>) -> Client + Send + 'static,
) -> Result<EmbeddedHost, HostError> {
    let size = options.size;
    let closing = Arc::new(AtomicBool::new(false));
    let ui_closing = closing.clone();
    let make = arvane::embed::driver(&options, move || {
        let client = client(ui_closing.clone());
        let frame_log = Rc::new(RefCell::new(FrameLog::from_env()));
        let log = frame_log.clone();
        let host = host::new(client, size, arvane::Engine::new(), frame_log)
            .with_embedded_closing(ui_closing);
        Embedder::new(host).on_renderer_change(move |change| {
            let mut log = log.borrow_mut();
            match change {
                arvane::RendererChange::Start { .. } => log.note(&format!("host=arvane {change}")),
                _ => log.note(&change.to_string()),
            }
        })
    });
    arvane_native::embed::open(parent, options, hooks, make)
        .map(Embedded::new)
        .map(|host| EmbeddedHost { host, closing })
        .map_err(|e| HostError::new(e.to_string()))
}
