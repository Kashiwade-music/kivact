//! Role: Embedded hosts using the desktop client's turn and rendering loop.

use crate::bridge::Client;
use crate::{FrameLog, host};
use arvane::embed::{Embedded, Embedder};
pub use arvane::{detached_threads, retain_module};
use kivact_wire::{HostError, Size};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use ui_types::{GpuPreference, ParentWindow};

/// Initial embedded host settings.
pub struct EmbedOptions {
    /// The host's parent window.
    pub parent: ParentWindow,
    /// Initial logical size, defaulting to 960 by 720.
    pub window_size: Option<Size>,
    /// Host-decided scale, or the OS scale.
    pub scale: Option<f32>,
    /// Ask the host for a physical content size.
    pub on_resize_request: Option<std::sync::Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
    /// GPU device preference (default when interop is disabled).
    pub gpu_preference: GpuPreference,
}

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
    pub fn set_scale(&self, scale: f32) {
        self.host.set_scale(scale);
    }
    /// Last applied scale.
    pub fn scale(&self) -> f32 {
        self.host.scale()
    }
    /// Why the host stopped, or `None` while running.
    pub fn failure(&self) -> Option<HostError> {
        self.host.failure().map(|e| HostError::new(e.to_string()))
    }
}

/// Start a client inside a host window; build the client on its UI thread.
pub fn embed_native(
    client: impl FnOnce() -> Client + Send + 'static,
    options: EmbedOptions,
) -> Result<EmbeddedHost, HostError> {
    let size = options.window_size.unwrap_or(Size {
        width: 960.0,
        height: 720.0,
    });
    let closing = Arc::new(AtomicBool::new(false));
    let ui_closing = closing.clone();
    arvane::embed::open(
        arvane::embed::EmbedOptions {
            parent: options.parent,
            size,
            scale: options.scale,
            on_resize_request: options.on_resize_request,
            prefer_gpu: true,
            gpu_preference: options.gpu_preference,
        },
        move || {
            let client = client();
            let frame_log = Rc::new(RefCell::new(FrameLog::from_env()));
            let log = frame_log.clone();
            let host = host::new(client, size, arvane::Engine::new(), frame_log)
                .with_embedded_closing(ui_closing);
            Embedder::new(host).on_renderer_change(move |change| {
                let mut log = log.borrow_mut();
                match change {
                    arvane::RendererChange::Start { .. } => {
                        log.note(&format!("host=arvane {change}"))
                    }
                    _ => log.note(&change.to_string()),
                }
            })
        },
    )
    .map(|host| EmbeddedHost { host, closing })
    .map_err(|e| HostError::new(e.to_string()))
}
