//! Role: Embedded hosts using the desktop client's turn and rendering loop.

use crate::api::Client;
use crate::contract::CloseSignal;
use crate::{FrameLog, host};
pub use arvane_native::retain_module;
pub use arvane_shell::{ChildHooks, ChildStop, EmbedNotice, PanicSlot};
use kivact_types::EmbedOptions;
use kivact_types::NativeWindow;
use std::cell::RefCell;
use std::rc::Rc;

/// An embedded engine host. `Send` and `Sync`. Drop requests closure and stops new turns.
pub struct EmbeddedHost {
    host: Box<dyn arvane_shell::ShellChild>,
    closing: CloseSignal,
    // The last field: it drops after the host, whose cleanup may detach threads.
    _retention: Retention,
}

/// Keeps the code module mapped when cleanup detached a thread that still runs.
struct Retention;
impl Drop for Retention {
    fn drop(&mut self) {
        retain_detached();
    }
}

/// Keep the module mapped while threads detached during cleanup still run.
fn retain_detached() {
    if kivact_types::process::DetachedThreads::any() {
        retain_module();
    }
}
impl Drop for EmbeddedHost {
    fn drop(&mut self) {
        // The UI may still be inside Host::effects. Stop its next client turn
        // before the lower native handle can defer destruction of this host.
        self.closing.close();
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
    /// Show or hide the window; queued.
    pub fn set_visibility(&self, visibility: kivact_types::embed::EmbedVisibility) {
        self.host.set_visibility(visibility);
    }
    /// Last applied visibility.
    pub fn visibility(&self) -> kivact_types::embed::EmbedVisibility {
        self.host.visibility()
    }
    /// Why the host stopped, or `None` while running.
    pub fn stop(&self) -> Option<ChildStop> {
        self.host.stop()
    }
}

/// Why an embedded host did not open.
#[derive(Debug)]
pub enum EmbedOpenError {
    /// This platform or build cannot embed a window.
    Unsupported(String),
    /// The platform failed to open it.
    Failed(String),
}

/// Start a client inside a host window; build the client on its UI thread
/// with the close flag the returned host sets when it is dropped (the client
/// stops delivering the rest of its event batch once it is set).
pub fn embed_native(
    parent: NativeWindow,
    options: EmbedOptions,
    hooks: ChildHooks,
    client: impl FnOnce(CloseSignal) -> Client + Send + 'static,
) -> Result<EmbeddedHost, EmbedOpenError> {
    let size = options.size;
    let closing = CloseSignal::new();
    let ui_closing = closing.clone();
    let make = arvane::embed_driver(options.gpu_preference, move || {
        let client = client(ui_closing.clone());
        let frame_log = Rc::new(RefCell::new(FrameLog::from_env()));
        let log = frame_log.clone();
        let host = host::new(client, size, arvane_text::TextEngine::new(), frame_log)
            .with_close(ui_closing);
        let observer = arvane::observer(crate::frame_log::renderer_logger(log));
        (host, Some(observer))
    });
    match arvane_native::embed::open(parent, options, hooks, make).map(|host| EmbeddedHost {
        host,
        closing,
        _retention: Retention,
    }) {
        Ok(host) => Ok(host),
        Err(e) => {
            // The failed open may have detached threads of its own.
            retain_detached();
            Err(if e.is_unsupported() {
                EmbedOpenError::Unsupported(e.to_string())
            } else {
                EmbedOpenError::Failed(e.to_string())
            })
        }
    }
}
