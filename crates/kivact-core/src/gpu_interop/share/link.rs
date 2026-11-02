//! Role: The engine's side of a shared image: its requests, textures and frames.
//!
//! `ShareLink` is the `Send + Sync` handle an engine's thread holds; the
//! state it shares with the UI side ([`LinkState`]) lives in an
//! [`Inbox`], so each change the engine makes wakes the UI task of the
//! shared image, and the engine's waits wake on the UI's changes.
//!
//! Invariants:
//! - `LinkState::Open` holds the newest request (its token is one more
//!   than the one before; a call naming another token is stale), its phase
//!   and the newest CPU picture; the UI side moves the phase, except
//!   `Waiting` → `Provided`, which `provide` does.
//! - Once `Closed`, nothing is kept: textures given are dropped, frames
//!   are not published, and the waits return at once.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use kivact_reactive::UiError;
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

/// Where a [`SharedImage`](super::app_side::SharedImage) stands, tracked
/// ([`SharedImage::status`](super::app_side::SharedImage::status)).
#[derive(Clone, Debug, PartialEq)]
pub enum ShareStatus {
    /// A [`ShareMode::Gpu`] request was issued; the engine has not
    /// provided textures for it yet.
    Waiting,
    /// The engine provided textures; the window imports them.
    Importing,
    /// The textures are imported: the engine's frames draw
    /// ([`SharedImage::frame`](super::app_side::SharedImage::frame)).
    Ready,
    /// The request is [`ShareMode::Pixels`]: the window cannot import, and
    /// the engine publishes pictures drawn on the CPU
    /// ([`SharedImage::pixels`](super::app_side::SharedImage::pixels)).
    Pixels,
    /// The window could not import the textures (the kinds of
    /// `ImportStatus::Failed`). The image stays so until the next request.
    Failed(UiError),
    /// The image's owner is gone: nothing draws, and the engine's link is
    /// closed (`ShareLink::is_closed`).
    Closed,
}

/// The state the engine's threads share with the UI side of the image.
pub(crate) enum LinkState {
    /// The image's owner lives.
    Open(Box<Open>),
    /// The image's owner is gone: nothing is kept, frames are not
    /// published and the waits return at once.
    Closed,
}

/// The state of an image whose owner lives.
pub(crate) struct Open {
    /// The newest request; its token is one more than the one before.
    request: ShareRequest,
    /// Where the newest request stands.
    phase: Phase,
    /// The newest picture published on the CPU, for the UI to take
    /// (whatever the request).
    pixels: Option<Pixels>,
}

/// Where the newest request stands. `Pixels` exactly for a request of
/// `ShareMode::Pixels` (`Phase::of`).
pub(crate) enum Phase {
    /// A GPU request waits for textures.
    Waiting,
    /// The engine gave textures; the UI has not taken them yet.
    Provided(SharedTextures),
    /// The UI imports the textures.
    Importing,
    /// Imported: frames are published through the import's publisher.
    Ready(FramePublisher),
    /// The import failed (the kinds of `ImportStatus::Failed`).
    Failed(UiError),
    /// A request of `ShareMode::Pixels`.
    Pixels,
}

/// What the engine handed the UI.
pub(crate) enum Work {
    /// Textures to import for the request of the token.
    Import(u64, SharedTextures),
    /// A picture drawn on the CPU.
    Pixels(Pixels),
}

impl LinkState {
    pub(super) fn new(request: ShareRequest) -> Self {
        let phase = Phase::of(&request.mode);
        LinkState::Open(Box::new(Open {
            request,
            phase,
            pixels: None,
        }))
    }

    /// Replace the request with a new one of `mode` and size; the old
    /// phase (its textures or publisher) is returned, to drop after the
    /// lock.
    pub(super) fn renew(&mut self, mode: ShareMode, (width, height): (u32, u32)) -> Option<Phase> {
        let LinkState::Open(open) = self else {
            return None;
        };
        let phase = std::mem::replace(&mut open.phase, Phase::of(&mode));
        open.request = ShareRequest {
            mode,
            width,
            height,
            token: open.request.token + 1,
        };
        Some(phase)
    }

    /// The owner is gone; what was kept is returned, to drop after the
    /// lock.
    pub(super) fn close(&mut self) -> LinkState {
        std::mem::replace(self, LinkState::Closed)
    }

    /// What the UI task takes: provided textures (the phase becomes
    /// `Importing`), else the newest picture.
    pub(super) fn take_work(&mut self) -> Option<Work> {
        let LinkState::Open(open) = self else {
            return None;
        };
        match std::mem::replace(&mut open.phase, Phase::Importing) {
            Phase::Provided(textures) => Some(Work::Import(open.request.token, textures)),
            other => {
                open.phase = other;
                open.pixels.take().map(Work::Pixels)
            }
        }
    }

    /// The import of request `token` settled as `phase` (`Ready` or
    /// `Failed`); whether it is the newest request's import.
    pub(super) fn settle(&mut self, token: u64, phase: Phase) -> bool {
        match self {
            LinkState::Open(open)
                if open.request.token == token && matches!(open.phase, Phase::Importing) =>
            {
                open.phase = phase;
                true
            }
            _ => false,
        }
    }

    /// The status the UI shows.
    pub(super) fn status(&self) -> ShareStatus {
        let LinkState::Open(open) = self else {
            return ShareStatus::Closed;
        };
        match &open.phase {
            Phase::Waiting => ShareStatus::Waiting,
            Phase::Provided(_) | Phase::Importing => ShareStatus::Importing,
            Phase::Ready(_) => ShareStatus::Ready,
            Phase::Failed(error) => ShareStatus::Failed(error.clone()),
            Phase::Pixels => ShareStatus::Pixels,
        }
    }

    /// The newest request, `None` once closed.
    pub(super) fn newest(&self) -> Option<&ShareRequest> {
        match self {
            LinkState::Open(open) => Some(&open.request),
            LinkState::Closed => None,
        }
    }

    /// Give the newest request `request` the textures; on a refusal, the
    /// error and the textures to drop after the lock.
    fn provide(
        &mut self,
        request: &ShareRequest,
        textures: SharedTextures,
    ) -> Result<(), (ShareError, SharedTextures)> {
        let LinkState::Open(open) = self else {
            return Err((ShareError::Closed, textures));
        };
        if open.request.token != request.token {
            return Err((ShareError::Stale, textures));
        }
        if matches!(open.phase, Phase::Waiting) {
            open.phase = Phase::Provided(textures);
            return Ok(());
        }
        let message = if matches!(open.phase, Phase::Pixels) {
            "a Pixels request takes no textures"
        } else {
            "the request was given textures already"
        };
        Err((ShareError::Failed(message.into()), textures))
    }

    /// The answer of `wait_ready` for `request`, `None` while it waits.
    fn ready(&self, request: &ShareRequest) -> Option<Result<(), ShareError>> {
        let LinkState::Open(open) = self else {
            return Some(Err(ShareError::Closed));
        };
        if open.request.token != request.token {
            return Some(Err(ShareError::Stale));
        }
        match &open.phase {
            Phase::Ready(_) => Some(Ok(())),
            Phase::Failed(error) => Some(Err(ShareError::Failed(error.message().to_owned()))),
            Phase::Pixels => Some(Err(ShareError::Failed(
                "a Pixels request imports nothing".into(),
            ))),
            Phase::Waiting | Phase::Provided(_) | Phase::Importing => None,
        }
    }

    /// The publisher of `request`'s import, while it is `Ready`.
    fn publisher(&self, request: &ShareRequest) -> Option<FramePublisher> {
        let LinkState::Open(open) = self else {
            return None;
        };
        match &open.phase {
            Phase::Ready(publisher) if open.request.token == request.token => {
                Some(publisher.clone())
            }
            _ => None,
        }
    }

    /// Keep `pixels` as the newest picture; the picture to drop (replaced,
    /// or refused when closed).
    fn put_pixels(&mut self, pixels: Pixels) -> Option<Pixels> {
        match self {
            LinkState::Closed => Some(pixels),
            LinkState::Open(open) => open.pixels.replace(pixels),
        }
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
        let request = self.state.read(|s| s.newest().cloned())?;
        self.take_if_new(request)
    }

    /// Wait at most `timeout` for a request newer than the last one this
    /// handle returned ([`ShareLink::poll_request`]); `None` when the time
    /// ran out, and at once when the link is closed.
    pub fn wait_request(&self, timeout: Duration) -> Option<ShareRequest> {
        let seen = self.seen.load(Ordering::SeqCst);
        let request = self
            .state
            .wait(timeout, |s| match s.newest() {
                None => Some(None),
                Some(newest) => (newest.token > seen).then(|| Some(newest.clone())),
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
        match self.state.update(|s| s.provide(request, textures)) {
            Ok(()) => Ok(()),
            Err((error, refused)) => {
                // The refused handles close outside the lock.
                drop(refused);
                Err(error)
            }
        }
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
            .wait(timeout, |s| s.ready(request))
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
        if let Some(publisher) = self.state.read(|s| s.publisher(request)) {
            publisher.publish(slot, value);
        }
    }

    /// Publish a picture drawn on the CPU: the UI thread wakes and
    /// `SharedImage::pixels` reads it. Taken
    /// whatever the request (the newest picture wins); ignored once
    /// closed.
    pub fn publish_pixels(&self, pixels: Pixels) {
        let dropped = self.state.update(|s| s.put_pixels(pixels));
        drop(dropped);
    }

    /// Whether the shared image's owner is gone: the engine stops.
    pub fn is_closed(&self) -> bool {
        self.state.read(|s| matches!(s, LinkState::Closed))
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
