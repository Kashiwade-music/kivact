//! Role: Public embedded App configuration and lifetime.
//!
//! Build a [`Parent`] from CLAP `clap_window.win32` or `clap_window.x11`,
//! VST3 `"HWND"` or `"X11EmbedWindowID"`, or nih-plug
//! `ParentWindowHandle::Win32Hwnd` or `ParentWindowHandle::X11Window`.
//! The App's initial `window_size` uses logical pixels; the host's `set_size`
//! and `size` use physical pixels. Scale is physical pixels per logical pixel.
//! Dropping [`Embedded`] requests closure without draining queued actions.
//! Keep plugin state outside the App so it survives closing and reopening.
//!
//! ```no_run
//! # #[cfg(feature = "embed")]
//! # {
//! use kivact_core::{App, view::text, embed::Parent};
//! # fn open(hwnd: *mut std::ffi::c_void) -> kivact_core::Result<()> {
//! let editor = App::new("Editor").window_size(640, 480)
//!     .embed(Parent::win32(hwnd).scale(1.0), || text("Editor"))?;
//! # Ok(())
//! # }
//! # }
//! ```

use crate::view::IntoView;
use crate::{App, Result, UiError};
use kivact_runtime::ErrorKind;
use kivact_types::ParentWindow;
use std::ffi::c_void;
use std::num::{NonZeroIsize, NonZeroU32};
use std::panic::Location;

/// A host window and optional host-decided scale. Constructors only store raw
/// values and never dereference them. `App::embed` validates the parent.
/// If the parent dies first, the child stops safely without undefined behavior.
#[derive(Clone)]
pub struct Parent {
    // xtask-allow(state-fields): optional validated constructor input, independent of parent scale.
    window: Option<ParentWindow>,
    // xtask-allow(state-fields): optional host scale configuration, independent of callback presence.
    scale: Option<f64>,
    on_resize_request: Option<std::sync::Arc<dyn Fn(u32, u32) -> bool + Send + Sync>>,
}
impl std::fmt::Debug for Parent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parent")
            .field("window", &self.window)
            .field("scale", &self.scale)
            .field("on_resize_request", &self.on_resize_request.is_some())
            .finish()
    }
}
impl Parent {
    /// A Win32 `HWND`: CLAP `clap_window.win32`, VST3 `"HWND"`,
    /// or nih-plug `ParentWindowHandle::Win32Hwnd`. Null is refused by `embed`.
    pub fn win32(hwnd: *mut c_void) -> Parent {
        Self {
            window: NonZeroIsize::new(hwnd as isize).map(ParentWindow::Win32),
            scale: None,
            on_resize_request: None,
        }
    }
    /// An X11 id: CLAP `clap_window.x11`, VST3 `"X11EmbedWindowID"`,
    /// or nih-plug `ParentWindowHandle::X11Window`.
    pub fn x11(window: u32) -> Parent {
        Self {
            window: NonZeroU32::new(window).map(ParentWindow::X11),
            scale: None,
            on_resize_request: None,
        }
    }
    /// Forward `platform::request_window_size` to the host, in physical pixels.
    /// Runs on the App UI thread and must not wait for the host GUI thread. Return
    /// true when the host accepted; Kivact then resizes the window unless the host
    /// called `Embedded::set_size` meanwhile, which wins. Without this callback,
    /// requests fail as `Unsupported`.
    pub fn on_resize_request(
        mut self,
        f: impl Fn(u32, u32) -> bool + Send + Sync + 'static,
    ) -> Parent {
        self.on_resize_request = Some(std::sync::Arc::new(f));
        self
    }
    /// Host-decided scale (CLAP `set_scale`, VST3 `setContentScaleFactor`).
    /// Without it the OS scale is used. Nonfinite or nonpositive values,
    /// including values outside positive finite f64 range, are refused by `embed`.
    pub fn scale(mut self, scale: f64) -> Parent {
        self.scale = Some(scale);
        self
    }
}

/// An App running inside a host window. `Send` and `Sync`.
/// Setters queue the change for the App UI thread and return at once; getters
/// return the last applied value. Sizes are physical pixels; scale is physical
/// pixels per logical pixel. A zero or oversized size, or a nonfinite or
/// nonpositive scale, is ignored with a warning. Prefer dropping on the host
/// GUI thread. Outside an App callback, dropping normally waits for cleanup of
/// the App and its native window. Each owned thread, including a thread running
/// UI callbacks, is waited for at most two seconds. After that limit, unfinished
/// cleanup may continue after drop returns; Kivact retains its code module so
/// the host may unload the plugin. Inside an App callback, the turn in progress
/// finishes and cleanup follows it; no later turn or input runs. Queued actions
/// are not drained.
/// After a platform failure the App stops, its window is destroyed and
/// `error` says why. A panic stops it the same way without running more App
/// code. It is not an error: the next call on `Embedded` (any method, or its
/// drop unless the thread is already panicking) resumes it on the calling
/// thread with its original payload. Do not let it unwind into a C-ABI host;
/// Rust aborts there. Keep plugin state outside the App.
/// If a thread exceeds its shutdown wait, or a native library needs its code
/// after closure, Kivact keeps its code module mapped until process exit. The
/// host may unload the plugin after closing the editor.
pub struct Embedded {
    // Fields drop in declaration order: stop the host before retaining code
    // for detached threads and resuming a captured panic.
    host: kivact_engine::embed::EmbeddedHost,
    guard: EmbeddedGuard,
}
impl Embedded {
    /// Set physical size (CLAP `set_size`, VST3 `onSize`). A zero or
    /// platform-oversized dimension is ignored with a warning.
    pub fn set_size(&self, width: u32, height: u32) {
        self.host.set_size(width, height);
        self.guard.resume();
    }
    /// Last applied physical size (CLAP `get_size`, VST3 `getSize`).
    pub fn size(&self) -> (u32, u32) {
        let size = self.host.size();
        self.guard.resume();
        size
    }
    /// Set physical pixels per logical pixel, keeping physical size until the
    /// host resizes it. To retain logical size, request logical size times scale
    /// from the host (CLAP `request_resize`, VST3 `resizeView`). A nonfinite or
    /// nonpositive scale is ignored with a warning.
    pub fn set_scale(&self, scale: f64) {
        self.host.set_scale(scale);
        self.guard.resume();
    }
    /// Last applied scale.
    pub fn scale(&self) -> f64 {
        let scale = self.host.scale();
        self.guard.resume();
        scale
    }
    /// Why the App stopped (a platform failure), or `None`.
    pub fn error(&self) -> Option<UiError> {
        let error = self
            .host
            .failure()
            .map(|e| UiError::new(ErrorKind::Platform, e.message, None));
        self.guard.resume();
        error
    }
}
type PanicPayload = Box<dyn std::any::Any + Send>;
enum EmbeddedState {
    Running,
    Panicked(PanicPayload),
    Closed,
}
struct EmbeddedGuard {
    panic: kivact_engine::embed::PanicSlot,
    state: std::sync::Mutex<EmbeddedState>,
}
impl EmbeddedGuard {
    fn resume(&self) {
        let mut state = self
            .state
            .lock()
            .expect("invariant: embedded state is not poisoned");
        if matches!(*state, EmbeddedState::Running)
            && let Some(payload) = self.panic.take()
        {
            *state = EmbeddedState::Panicked(payload);
        }
        if matches!(*state, EmbeddedState::Panicked(_)) {
            let EmbeddedState::Panicked(payload) =
                std::mem::replace(&mut *state, EmbeddedState::Closed)
            else {
                unreachable!("invariant: panicked state contains the original payload")
            };
            drop(state);
            std::panic::resume_unwind(payload);
        }
    }
}
impl Drop for EmbeddedGuard {
    fn drop(&mut self) {
        if kivact_types::process::DetachedThreads::any() {
            kivact_engine::embed::retain_module();
        }
        if !std::thread::panicking() {
            self.resume();
        }
    }
}

impl App {
    /// Run `root` inside `parent` and return once it is built (feature `embed`).
    /// Call on the host GUI thread. `root` and every App callback run on the App
    /// UI thread, which may be another thread; they must not wait for the host GUI
    /// thread. The initial physical size is `window_size` times the scale (default
    /// 960 by 720 logical pixels). Invalid sizes or scale return
    /// `ErrorKind::Message`; an invalid parent, unavailable OS resources or an
    /// unsupported OS return `ErrorKind::Platform`. A failed open either creates
    /// no window or cleans up a child already created. A panic in `root` or
    /// while opening is resumed from `embed`; a later panic stops the App at
    /// once (see [`Embedded`]). Startup failure normally waits for cleanup, with
    /// the same two-second limit per owned thread as drop; unfinished cleanup
    /// may continue under module retention after `Err` is returned. Title and
    /// fullscreen have no effect.
    /// `min_window_size` clamps `platform::request_window_size`, not the host.
    #[track_caller]
    #[cfg(feature = "embed")]
    pub fn embed<V: IntoView>(
        self,
        parent: Parent,
        root: impl FnOnce() -> V + Send + 'static,
    ) -> Result<Embedded> {
        self.embed_impl(parent, root, Location::caller())
    }

    #[track_caller]
    pub(crate) fn embed_impl<V: IntoView>(
        self,
        parent: Parent,
        root: impl FnOnce() -> V + Send + 'static,
        at: &'static Location<'static>,
    ) -> Result<Embedded> {
        self.options.check_window_sizes()?;
        if parent
            .scale
            .is_some_and(|scale| !scale.is_finite() || scale <= 0.0)
        {
            return Err(UiError::new(
                ErrorKind::Message,
                "Parent::scale must be finite and positive in f64",
                Some(at),
            ));
        }
        let window = parent.window.ok_or_else(|| {
            UiError::new(
                ErrorKind::Platform,
                "the parent window handle is null",
                Some(at),
            )
        })?;
        let panic = kivact_engine::embed::PanicSlot::default();
        let options = kivact_types::EmbedOptions {
            size: self
                .options
                .host_options()
                .window_size
                .unwrap_or(kivact_types::defaults::DEFAULT_WINDOW_SIZE),
            scale: parent.scale,
            prefer_gpu: true,
            gpu_preference: self.options.host_options().gpu_preference,
        };
        let hooks = kivact_engine::embed::ChildHooks {
            on_resize_request: parent.on_resize_request,
            panic: panic.clone(),
        };
        let opened = kivact_engine::embed::embed_native(window, options, hooks, move |closing| {
            crate::launch::Launch::new(
                self.options,
                crate::launch::ClientMode::Embedded(closing),
                move || root().into_view(),
            )
            .into_engine_client()
            .0
        });
        match opened {
            Ok(host) => {
                let embedded = Embedded {
                    host,
                    guard: EmbeddedGuard {
                        panic: panic.clone(),
                        state: std::sync::Mutex::new(EmbeddedState::Running),
                    },
                };
                // A pending payload unwinds from here; dropping `embedded`
                // closes the host first.
                panic.resume();
                Ok(embedded)
            }
            Err(e) => {
                panic.resume();
                Err(UiError::new(ErrorKind::Platform, e.message, None))
            }
        }
    }
}
