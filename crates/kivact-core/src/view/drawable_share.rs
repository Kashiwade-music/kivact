//! Role: The app's side of a drawable's export: requests, frames, textures.
//!
//! `DrawableShare` is the `Send + Sync` handle an app's render thread holds.
//! Its state (`ExportLink`) lives in an [`Inbox`]: each call wakes the UI
//! task of the drawable, which wakes the drawable's turn so the request
//! reaches the engine. Frames and textures come back from the engine on
//! the UI thread and are published to the waiting threads.
//!
//! Invariants:
//! - Each request starts a new generation (`ExportOpen::restart` makes it the
//!   current request's generation plus one, so none is reused); a frame or
//!   textures of another generation are dropped on arrival.
//! - Frame numbers count up over the drawable's whole life, so `wait(after)`
//!   of an older generation returns the first frame of the current one.
//! - `wait` never returns `ShareError::Stale`.

use std::sync::Arc;
use std::time::Duration;

use kivact_engine::contract::ExportRequestMode;
use kivact_types::Pixels;
use kivact_types::cell::Mailbox;
use kivact_types::geometry3d::Matrix4;
use kivact_types::render::{ExportFrame, ExportedFrame};
#[cfg(feature = "gpu-interop")]
use kivact_types::{Failure, GpuAdapter, SharedTextures};

use crate::sync::Inbox;
use crate::view::ShareError;

/// Send + Sync; usable from the app's render thread.
#[derive(Clone)]
pub struct DrawableShare {
    pub(crate) inner: Arc<Inbox<ExportLink>>,
}

/// What `DrawableShare::wait` returns.
#[derive(Debug)]
pub enum ShareUpdate {
    /// Shared textures to import before the frames of their generation.
    #[cfg(feature = "gpu-interop")]
    Textures(DrawableTextures),
    /// The newest frame of the current generation after the awaited number.
    Frame(DrawableFrame),
}

/// Textures the engine made for the current generation (GPU export).
#[cfg(feature = "gpu-interop")]
#[derive(Debug)]
pub struct DrawableTextures {
    /// The request these textures answer.
    pub generation: u64,
    /// The pool's textures, to import.
    pub textures: SharedTextures,
}

/// A frame of a drawable's export.
#[derive(Clone, Debug)]
pub enum DrawableFrame {
    /// A frame read back as pixels (straight-alpha RGBA8).
    Pixels {
        /// The request this frame answers.
        generation: u64,
        /// The frame number over the drawable's life.
        number: u64,
        /// The frame's pixels.
        pixels: Pixels,
    },
    /// The frame is ready in a slot of the shared pool at `value` on the ready fence.
    Gpu {
        /// The request this frame answers.
        generation: u64,
        /// The frame number over the drawable's life.
        number: u64,
        /// The pool slot holding the frame.
        slot: u32,
        /// The fence value that signals the frame is ready.
        value: u64,
    },
}

impl DrawableFrame {
    /// The frame number over the drawable's whole life.
    pub fn number(&self) -> u64 {
        match self {
            Self::Pixels { number, .. } | Self::Gpu { number, .. } => *number,
        }
    }

    /// The request this frame answers.
    pub fn generation(&self) -> u64 {
        match self {
            Self::Pixels { generation, .. } | Self::Gpu { generation, .. } => *generation,
        }
    }
}

/// The state of a drawable's export, shared by the app's threads and the UI.
pub(crate) enum ExportLink {
    Open(Box<ExportOpen>),
    /// The drawable was disposed.
    Closed,
}

/// The state of an open export.
pub(crate) struct ExportOpen {
    /// The current request, its generation and, for the requests that
    /// receive frames, the newest frame.
    pub(crate) request: ExportRequest,
    pub(crate) geometry: Mailbox<Option<Matrix4>>,
    // xtask-allow(state-fields): The app's configured scale; a setting, not derived from the window.
    pub(crate) scale: Option<f64>,
    // xtask-allow(state-fields): ID allocator: the last frame number issued over the drawable's life.
    pub(crate) next_number: u64,
    /// Take-once wake of the UI task: an app call the UI turn has not taken yet.
    pub(crate) wake: Mailbox<()>,
}

/// The export request in force, with its generation.
pub(crate) enum ExportRequest {
    /// Nothing is exported.
    None { generation: u64 },
    /// Frames are read back as pixels; `latest` is the newest one received.
    Pixels {
        // xtask-allow(state-fields): S-8 ID - the generation of the app's request; a frame of another generation is dropped
        generation: u64,
        // xtask-allow(state-fields): delivery record - the newest frame received for this request
        latest: Option<DrawableFrame>,
    },
    /// The window has to make the pool; no GPU frames yet.
    #[cfg(feature = "gpu-interop")]
    GpuRequested {
        generation: u64,
        adapter: GpuAdapter,
    },
    /// The window has no GPU device; retried on `GpuExportAvailable`.
    #[cfg(feature = "gpu-interop")]
    GpuWaiting {
        generation: u64,
        adapter: GpuAdapter,
    },
    /// Textures made. `textures` is taken once by `wait`; `latest` is the newest GPU frame.
    #[cfg(feature = "gpu-interop")]
    GpuProvided {
        // xtask-allow(state-fields): S-8 ID - the generation of the app's request; a frame of another generation is dropped
        generation: u64,
        adapter: GpuAdapter,
        /// Take-once hand-over of the pool to `wait` (S-9 Mailbox, not an `Option` that is taken).
        textures: Mailbox<SharedTextures>,
        // xtask-allow(state-fields): delivery record - the newest frame received for this request
        latest: Option<DrawableFrame>,
    },
    /// The request failed; `wait` reports it.
    #[cfg(feature = "gpu-interop")]
    Failed { generation: u64, message: String },
}

impl ExportRequest {
    /// The generation of this request.
    pub(crate) fn generation(&self) -> u64 {
        match self {
            Self::None { generation } | Self::Pixels { generation, .. } => *generation,
            #[cfg(feature = "gpu-interop")]
            Self::Failed { generation, .. } => *generation,
            #[cfg(feature = "gpu-interop")]
            Self::GpuRequested { generation, .. }
            | Self::GpuWaiting { generation, .. }
            | Self::GpuProvided { generation, .. } => *generation,
        }
    }
}

impl ExportOpen {
    /// A new generation (the current one plus one) with `request` made from it;
    /// the latest frame stays with the old request.
    fn restart(&mut self, request: impl FnOnce(u64) -> ExportRequest) {
        let generation = self.request.generation() + 1;
        self.request = request(generation);
    }

    /// The newest frame of the current request, when it receives frames.
    fn latest(&self) -> Option<&DrawableFrame> {
        match &self.request {
            ExportRequest::Pixels { latest, .. } => latest.as_ref(),
            #[cfg(feature = "gpu-interop")]
            ExportRequest::GpuProvided { latest, .. } => latest.as_ref(),
            _ => None,
        }
    }

    /// The generation the current request answers.
    pub(crate) fn current(&self) -> u64 {
        self.request.generation()
    }

    /// The engine's mode for the request in force: GPU frames only once the
    /// textures are made (`GpuRequested` and `GpuProvided` are asked of the
    /// window as `Gpu`).
    pub(crate) fn request_mode(&self) -> ExportRequestMode {
        match &self.request {
            ExportRequest::None { .. } => ExportRequestMode::None,
            #[cfg(feature = "gpu-interop")]
            ExportRequest::Failed { .. } => ExportRequestMode::None,
            ExportRequest::Pixels { .. } => ExportRequestMode::Pixels,
            #[cfg(feature = "gpu-interop")]
            ExportRequest::GpuWaiting { .. } => ExportRequestMode::None,
            #[cfg(feature = "gpu-interop")]
            ExportRequest::GpuRequested { adapter, .. }
            | ExportRequest::GpuProvided { adapter, .. } => ExportRequestMode::Gpu(adapter.clone()),
        }
    }

    /// Take the textures waiting to be imported, once.
    #[cfg(feature = "gpu-interop")]
    fn take_textures(&mut self) -> Option<DrawableTextures> {
        match &mut self.request {
            ExportRequest::GpuProvided {
                generation,
                textures,
                ..
            } => textures.take().map(|textures| DrawableTextures {
                generation: *generation,
                textures,
            }),
            _ => None,
        }
    }

    /// The engine answered the textures request of `generation`.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn on_textures(&mut self, generation: u64, result: Result<SharedTextures, Failure>) {
        if generation != self.current() {
            return;
        }
        let ExportRequest::GpuRequested { adapter, .. } = &self.request else {
            return;
        };
        let adapter = adapter.clone();
        match result {
            Ok(textures) => {
                self.request = ExportRequest::GpuProvided {
                    generation,
                    adapter,
                    textures: {
                        let slot = Mailbox::new();
                        slot.put(textures);
                        slot
                    },
                    latest: None,
                };
            }
            Err(failure) if failure.kind == kivact_types::FailureKind::NotReady => {
                self.request = ExportRequest::GpuWaiting {
                    generation,
                    adapter,
                };
            }
            Err(failure) => {
                self.request = ExportRequest::Failed {
                    generation,
                    message: failure.detail,
                };
            }
        }
    }

    /// The window's GPU export became available: a waiting request is made again.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn on_gpu_available(&mut self) {
        if let ExportRequest::GpuWaiting { adapter, .. } = &self.request {
            let adapter = adapter.clone();
            self.restart(|generation| ExportRequest::GpuRequested {
                generation,
                adapter,
            });
        }
    }

    /// The pool was lost or the drawable was resized: the GPU request is made again.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn on_gpu_renew(&mut self, generation: u64) {
        if generation != self.current() {
            return;
        }
        if let ExportRequest::GpuProvided { adapter, .. } = &self.request {
            let adapter = adapter.clone();
            self.restart(|generation| ExportRequest::GpuRequested {
                generation,
                adapter,
            });
        }
    }

    /// Accept a frame the compositor exported when its generation is current
    /// and the current request receives frames.
    pub(crate) fn on_frame(&mut self, frame: ExportedFrame) {
        if frame.generation != self.current() {
            return;
        }
        let latest = match &mut self.request {
            ExportRequest::Pixels { latest, .. } => latest,
            #[cfg(feature = "gpu-interop")]
            ExportRequest::GpuProvided { latest, .. } => latest,
            _ => return,
        };
        self.next_number += 1;
        let (generation, number) = (frame.generation, self.next_number);
        *latest = Some(match frame.frame {
            ExportFrame::Pixels(pixels) => DrawableFrame::Pixels {
                generation,
                number,
                pixels,
            },
            ExportFrame::Gpu { slot, value } => DrawableFrame::Gpu {
                generation,
                number,
                slot,
                value,
            },
        });
    }
}

impl ExportLink {
    /// The open state, or `None` when disposed.
    fn open_mut(&mut self) -> Option<&mut ExportOpen> {
        match self {
            Self::Open(open) => Some(open.as_mut()),
            Self::Closed => None,
        }
    }

    /// The initial state of a new drawable's export: nothing requested.
    pub(crate) fn initial() -> Self {
        Self::Open(Box::new(ExportOpen {
            request: ExportRequest::None { generation: 0 },
            geometry: Mailbox::default(),
            scale: None,
            next_number: 0,
            wake: Mailbox::default(),
        }))
    }

    /// Take the wake marker of the UI side; `None` when nothing changed.
    pub(crate) fn take_changed(&mut self) -> Option<()> {
        self.open_mut()?.wake.take()
    }
}

impl DrawableShare {
    /// Change the link with `f` and wake the UI task of the drawable.
    fn change(&self, f: impl FnOnce(&mut ExportLink)) {
        self.inner.update(|link| {
            f(link);
            if let Some(open) = link.open_mut() {
                open.wake.put(());
            }
        });
    }

    /// Receive frames as CPU pixels (straight-alpha RGBA8). Starts a new generation.
    pub fn request_pixels(&self) {
        self.change(|link| {
            if let Some(open) = link.open_mut() {
                open.restart(|generation| ExportRequest::Pixels {
                    generation,
                    latest: None,
                });
            }
        });
    }

    /// Receive frames in textures shared with the app's device on `adapter`
    /// (the window's adapter: `kivact::gpu_interop::adapter()`). Starts a new generation.
    #[cfg(feature = "gpu-interop")]
    pub fn request_gpu(&self, adapter: GpuAdapter) {
        self.change(|link| {
            if let Some(open) = link.open_mut() {
                open.restart(|generation| ExportRequest::GpuRequested {
                    generation,
                    adapter,
                });
            }
        });
    }

    /// Stop exporting (frees the window's pool). Starts a new generation.
    pub fn stop(&self) {
        self.change(|link| {
            if let Some(open) = link.open_mut() {
                open.restart(|generation| ExportRequest::None { generation });
            }
        });
    }

    /// Device px per logical px of exported frames (default: the window's scale).
    pub fn set_scale(&self, scale: f64) {
        self.change(|link| {
            if let Some(open) = link.open_mut() {
                open.scale = Some(scale);
            }
        });
    }

    /// Where the app shows the drawable: drawable box (logical px) → the
    /// hosting viewport's local logical px; None hides it from hit testing.
    pub fn set_geometry(&self, geometry: Option<Matrix4>) {
        self.inner.read(|link| {
            if let ExportLink::Open(open) = link {
                open.geometry.put(geometry);
            }
        });
        self.change(|_| {});
    }

    /// The newest frame of the current generation, if any.
    pub fn latest(&self) -> Option<DrawableFrame> {
        self.inner.read(|link| match link {
            ExportLink::Open(open) => open.latest().cloned(),
            ExportLink::Closed => None,
        })
    }

    /// Wait for what the consumer must handle next:
    /// - new shared textures not yet taken (after `request_gpu`, a resize or
    ///   a device change): import them; GPU frames of that generation come
    ///   only after the import;
    /// - otherwise a frame of the current generation with `number > after`.
    ///
    /// Errors: `Timeout`, `Closed` (disposed), `Failed` (the request failed).
    /// Never `Stale`.
    pub fn wait(&self, after: u64, timeout: Duration) -> Result<ShareUpdate, ShareError> {
        self.inner
            .wait(timeout, |link| take_update(link, after))
            .unwrap_or(Err(ShareError::Timeout))
    }

    /// Whether the drawable was disposed.
    pub fn is_closed(&self) -> bool {
        self.inner.read(|link| matches!(link, ExportLink::Closed))
    }
}

/// What a waiting consumer can take now, if anything.
fn take_update(link: &mut ExportLink, after: u64) -> Option<Result<ShareUpdate, ShareError>> {
    let open = match link {
        ExportLink::Closed => return Some(Err(ShareError::Closed)),
        ExportLink::Open(open) => open,
    };
    #[cfg(feature = "gpu-interop")]
    if let Some(textures) = open.take_textures() {
        return Some(Ok(ShareUpdate::Textures(textures)));
    }
    #[cfg(feature = "gpu-interop")]
    if let ExportRequest::Failed { message, .. } = &open.request {
        return Some(Err(ShareError::Failed(message.clone())));
    }
    match open.latest() {
        Some(frame) if frame.number() > after => Some(Ok(ShareUpdate::Frame(frame.clone()))),
        _ => None,
    }
}
