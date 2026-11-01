//! Role: The engine's side of a shared image: its requests, textures and frames.
//!
//! `ShareLink` is the `Send + Sync` handle an engine's thread holds; the
//! state it shares with the UI side ([`LinkState`]) lives in an
//! [`Inbox`], so each change the engine makes wakes the UI task of the
//! shared image, and the engine's waits wake on the UI's changes.
//!
//! Invariants:
//! - `LinkState::request` is the newest request; its token is one more
//!   than the one before. A call naming another token is stale.
//! - `LinkState::phase` is the phase of the newest request only; the UI
//!   side moves it, except `Waiting` → `Importing`, which `provide` does.
//! - Once `closed`, nothing is kept: textures given are dropped, frames
//!   are not published, and the waits return at once.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use kivact_wire::{GpuAdapter, SharedTextures};

use crate::gpu_interop::import::FramePublisher;
use crate::gpu_interop::import::inbox::Inbox;
use kivact_style::draw::Pixels;

/// How the engine shares its pictures for a [`ShareRequest`].
#[derive(Clone, Debug, PartialEq)]
pub enum ShareMode {
    /// Make the engine's device on this adapter (with its graphics API),
    /// a pool of textures of the request's size, and
    /// [`ShareLink::provide`] them; then [`ShareLink::publish`] frames.
    Gpu(GpuAdapter),
    /// The window cannot import textures (it draws on the CPU, its GPU
    /// device is not ready, or the platform imports nothing): draw on the
    /// CPU and [`ShareLink::publish_pixels`].
    Pixels,
}

/// What the UI asks of the engine: how to share, and the size of the
/// pictures in pixels. Each request replaces the one before it; a call
/// that names an older one is [`ShareError::Stale`].
#[derive(Clone, Debug, PartialEq)]
pub struct ShareRequest {
    /// How to share.
    pub mode: ShareMode,
    /// The width in pixels of the pool's textures.
    pub width: u32,
    /// The height in pixels of the pool's textures.
    pub height: u32,
    /// Counted from 1, one more for each request of the image.
    pub(crate) token: u64,
}

/// Why a call of a [`ShareLink`] did not do what it asked.
#[derive(Clone, Debug, PartialEq)]
pub enum ShareError {
    /// The request was replaced by a newer one: drop what was made for
    /// it and take the newer one ([`ShareLink::poll_request`]).
    Stale,
    /// The shared image's owner is gone: stop.
    Closed,
    /// The wait ran out of time.
    Timeout,
    /// The request cannot be served so: the window refused the textures
    /// (its message), a [`ShareMode::Pixels`] request was given textures,
    /// or a request was given textures twice.
    Failed(String),
}

impl fmt::Display for ShareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShareError::Stale => f.write_str("the share request was replaced by a newer one"),
            ShareError::Closed => f.write_str("the shared image is closed"),
            ShareError::Timeout => f.write_str("the wait for the shared image timed out"),
            ShareError::Failed(message) => write!(f, "the shared image failed: {message}"),
        }
    }
}

impl std::error::Error for ShareError {}

/// Where the newest request stands.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Phase {
    /// A GPU request waits for textures.
    Waiting,
    /// Textures were given and are being imported.
    Importing,
    /// Imported: frames are published.
    Ready,
    /// The import failed, with the window's message.
    Failed(String),
    /// A request of `ShareMode::Pixels`.
    Pixels,
}

/// The state the engine's threads share with the UI side of the image.
pub(crate) struct LinkState {
    pub(crate) request: ShareRequest,
    pub(crate) phase: Phase,
    /// Textures given for the request of the token, for the UI to import.
    pub(crate) provided: Option<(u64, SharedTextures)>,
    /// The newest picture published on the CPU, for the UI to take.
    pub(crate) pixels: Option<Pixels>,
    /// The publisher of the newest request's import, once it is `Ready`.
    pub(crate) publisher: Option<FramePublisher>,
    /// The owner of the image is gone.
    pub(crate) closed: bool,
}

impl LinkState {
    pub(crate) fn new(request: ShareRequest) -> Self {
        let phase = Phase::of(&request.mode);
        LinkState {
            request,
            phase,
            provided: None,
            pixels: None,
            publisher: None,
            closed: false,
        }
    }

    /// Replace the request with a new one of `mode` and size; what was
    /// given for the old one is returned, to drop after the lock.
    pub(crate) fn renew(
        &mut self,
        mode: ShareMode,
        (width, height): (u32, u32),
    ) -> Option<(u64, SharedTextures)> {
        self.phase = Phase::of(&mode);
        self.request = ShareRequest {
            mode,
            width,
            height,
            token: self.request.token + 1,
        };
        self.publisher = None;
        self.provided.take()
    }

    /// Whether `request` is the newest one.
    fn is_current(&self, request: &ShareRequest) -> bool {
        self.request.token == request.token
    }
}

impl Phase {
    /// The phase a new request of `mode` starts in.
    fn of(mode: &ShareMode) -> Phase {
        match mode {
            ShareMode::Gpu(_) => Phase::Waiting,
            ShareMode::Pixels => Phase::Pixels,
        }
    }
}

/// The engine's handle of a `SharedImage`
/// (`SharedImage::link`): `Send + Sync +
/// Clone`, for the engine's thread.
///
/// The engine takes the newest [`ShareRequest`] ([`ShareLink::poll_request`],
/// [`ShareLink::wait_request`]) and serves it: for [`ShareMode::Gpu`], a
/// pool of textures on that adapter, [`ShareLink::provide`]d and, once
/// [`ShareLink::wait_ready`] is `Ok`, drawn and [`ShareLink::publish`]ed
/// frame after frame; for [`ShareMode::Pixels`], pictures drawn on the CPU
/// and [`ShareLink::publish_pixels`]ed. When a newer request comes, the
/// engine drops the pool of the old one (the UI has already dropped its
/// import) and serves the new one.
///
/// Every call that changes the image wakes the UI thread through its
/// executor, as a worker's channel wakes a task: Kivact runs no thread of
/// the engine's.
pub struct ShareLink {
    state: Arc<Inbox<LinkState>>,
    /// The token of the newest request this handle returned.
    seen: AtomicU64,
}

impl ShareLink {
    pub(crate) fn new(state: Arc<Inbox<LinkState>>) -> Self {
        ShareLink {
            state,
            seen: AtomicU64::new(0),
        }
    }

    /// The newest request, when it is newer than the last one this handle
    /// returned; `None` otherwise and once closed.
    pub fn poll_request(&self) -> Option<ShareRequest> {
        let request = self
            .state
            .read(|s| (!s.closed).then(|| s.request.clone()))?;
        self.take_if_new(request)
    }

    /// Wait at most `timeout` for a request newer than the last one this
    /// handle returned ([`ShareLink::poll_request`]); `None` when the time
    /// ran out, and at once when the link is closed.
    pub fn wait_request(&self, timeout: Duration) -> Option<ShareRequest> {
        let seen = self.seen.load(Ordering::SeqCst);
        let request = self
            .state
            .wait(timeout, |s| {
                if s.closed {
                    Some(None)
                } else {
                    (s.request.token > seen).then(|| Some(s.request.clone()))
                }
            })
            .flatten()?;
        self.take_if_new(request)
    }

    fn take_if_new(&self, request: ShareRequest) -> Option<ShareRequest> {
        let before = self.seen.fetch_max(request.token, Ordering::SeqCst);
        (request.token > before).then_some(request)
    }

    /// Hand the UI the pool made for `request`, a [`ShareMode::Gpu`]
    /// request: the UI thread imports it into the window (as
    /// `gpu_interop::import` does), and [`ShareLink::wait_ready`] tells
    /// when it is ready. Only the newest request takes textures, once.
    ///
    /// # Errors
    ///
    /// [`ShareError::Stale`] when a newer request replaced `request`,
    /// [`ShareError::Closed`] once the image's owner is gone, and
    /// [`ShareError::Failed`] for a [`ShareMode::Pixels`] request or one
    /// given textures before. A refused pool is dropped (its handles
    /// closed).
    ///
    /// # Safety
    ///
    /// As for `gpu_interop::import`: the engine keeps the contract of
    /// `SharedTextures` (its rustdoc, "The engine's contract"): how the
    /// textures and fences are made, on the request's adapter and size,
    /// the hand-off state, writing a texture only after
    /// [`ShareLink::wait_ready`] is `Ok` and only once the release fence
    /// reached the value of its last frame, and new textures for a new
    /// request. Breaking it is undefined behavior of the graphics API
    /// (Vulkan and Direct3D 12): a broken picture, a lost device, a
    /// crashed process, or access to GPU memory outside the resource are
    /// all possible. Kivact and Arvane cannot check most of it, which is
    /// why this is an `unsafe fn`.
    #[allow(
        unsafe_code,
        reason = "the engine's side of GPU texture sharing is a contract the caller keeps"
    )]
    pub unsafe fn provide(
        &self,
        request: &ShareRequest,
        textures: SharedTextures,
    ) -> Result<(), ShareError> {
        let (result, refused) = self.state.update(|s| {
            let refusal = if s.closed {
                ShareError::Closed
            } else if !s.is_current(request) {
                ShareError::Stale
            } else if s.request.mode == ShareMode::Pixels {
                ShareError::Failed("a Pixels request takes no textures".into())
            } else if s.phase != Phase::Waiting {
                ShareError::Failed("the request was given textures already".into())
            } else {
                s.phase = Phase::Importing;
                s.provided = Some((request.token, textures));
                return (Ok(()), None);
            };
            (Err(refusal), Some(textures))
        });
        // The refused handles close outside the lock.
        drop(refused);
        result
    }

    /// Wait at most `timeout` for the import of the textures given for
    /// `request` ([`ShareLink::provide`]) to be ready; the engine draws
    /// into them only after `Ok`.
    ///
    /// # Errors
    ///
    /// [`ShareError::Failed`] with the window's message when the import
    /// failed (or for a [`ShareMode::Pixels`] request),
    /// [`ShareError::Stale`] when a newer request replaced `request`,
    /// [`ShareError::Closed`] at once when the image's owner is gone, and
    /// [`ShareError::Timeout`] when the time ran out.
    pub fn wait_ready(&self, request: &ShareRequest, timeout: Duration) -> Result<(), ShareError> {
        self.state
            .wait(timeout, |s| {
                if s.closed {
                    return Some(Err(ShareError::Closed));
                }
                if !s.is_current(request) {
                    return Some(Err(ShareError::Stale));
                }
                match &s.phase {
                    Phase::Ready => Some(Ok(())),
                    Phase::Failed(message) => Some(Err(ShareError::Failed(message.clone()))),
                    Phase::Pixels => Some(Err(ShareError::Failed(
                        "a Pixels request imports nothing".into(),
                    ))),
                    Phase::Waiting | Phase::Importing => None,
                }
            })
            .unwrap_or(Err(ShareError::Timeout))
    }

    /// Publish the frame the engine drew for `request` into the pool's
    /// texture `slot` and signaled ready with `value`: the UI thread wakes
    /// and `SharedImage::frame` reads it.
    ///
    /// The newest frame wins: frames published before the UI thread took
    /// the last one are skipped, which is safe for the pool (Arvane
    /// signals the release fence to the value it drew, less one, so the
    /// skipped frames are released with the drawn one). Ignored when
    /// `request` is not the newest, not ready yet
    /// ([`ShareLink::wait_ready`]), a [`ShareMode::Pixels`] request, and
    /// once closed.
    pub fn publish(&self, request: &ShareRequest, slot: u32, value: u64) {
        let publisher = self.state.read(|s| {
            if s.closed || !s.is_current(request) || s.phase != Phase::Ready {
                return None;
            }
            s.publisher.clone()
        });
        if let Some(publisher) = publisher {
            publisher.publish(slot, value);
        }
    }

    /// Publish a picture drawn on the CPU: the UI thread wakes and
    /// `SharedImage::pixels` reads it. Taken
    /// whatever the request (the newest picture wins); ignored once
    /// closed.
    pub fn publish_pixels(&self, pixels: Pixels) {
        let dropped = self.state.update(|s| {
            if s.closed {
                Some(pixels)
            } else {
                s.pixels.replace(pixels)
            }
        });
        drop(dropped);
    }

    /// Whether the shared image's owner is gone: the engine stops.
    pub fn is_closed(&self) -> bool {
        self.state.read(|s| s.closed)
    }
}

impl Clone for ShareLink {
    /// Another handle of the same image, which has returned the same
    /// requests as this one.
    fn clone(&self) -> Self {
        ShareLink {
            state: self.state.clone(),
            seen: AtomicU64::new(self.seen.load(Ordering::SeqCst)),
        }
    }
}

impl fmt::Debug for ShareLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShareLink")
            .field("closed", &self.is_closed())
            .finish_non_exhaustive()
    }
}
