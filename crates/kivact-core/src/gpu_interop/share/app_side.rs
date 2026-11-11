//! Role: The App's side of a shared image: status, size, renewal and import.

use std::cell::RefCell;
use std::fmt;
use std::panic::Location;
use std::sync::Arc;

use kivact_runtime::{Notifier, Owner, Runtime, Signal};
use kivact_types::cell::Sent;
use kivact_types::{GpuAdapter, SharedTextures};

use super::link::{LinkState, Phase, ShareLink, ShareMode, ShareRequest, ShareStatus, Work};
use crate::gpu_interop::adapter;
use crate::gpu_interop::import::{ExternalImage, ImportStatus, import_in};
use crate::sync::Inbox;
use kivact_runtime::spawn_in;
use kivact_style::draw::{GpuFrame, Pixels};

/// An image an engine on another thread shares with the UI
/// ([`shared_image`]): the UI thread's handle, `Clone` and `!Send`.
///
/// Hand the engine its [`SharedImage::link`]; draw [`SharedImage::frame`]
/// with `Draw::gpu_frame`, and [`SharedImage::pixels`] with `Draw::pixels`
/// when there is no frame.
#[derive(Clone)]
pub struct SharedImage {
    state: Arc<Inbox<LinkState>>,
    /// Notified after each change of `state` the UI makes or takes from the
    /// engine (the receiving task): [`SharedImage::status`] tracks it.
    changed: Notifier,
    // xtask-allow(state-fields): projection: the import of the link's current request, written only by SharedImage::replace_current
    /// The import of the newest request, once the engine provided it.
    current: Signal<Option<Current>>,
    // xtask-allow(state-fields): independent: the last pixels delivered, kept for every phase
    pixels: Signal<Option<Pixels>>,
    size: Signal<(u32, u32)>,
}

/// The import of a request: the image and its owner.
#[derive(Clone)]
struct Current {
    token: u64,
    owner: Owner,
    image: ExternalImage,
}

impl PartialEq for Current {
    /// One import per request.
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

/// An image an engine on another thread shares with the window, of
/// `width` × `height` pixels (at least 1), belonging to the setup owner.
/// Setup only (a located `SetupOnly` misuse elsewhere).
///
/// It does for the App what the low-level import leaves to it. It issues
/// a [`ShareRequest`] at once and whenever it is out of date: the adapter
/// ([`adapter`]) changed (its generation, or `Some` and `None`),
/// [`SharedImage::set_size`] asked another size, or the import was `Lost`.
/// The request is [`ShareMode::Gpu`] with the adapter when there is one,
/// [`ShareMode::Pixels`] when not. Each request has a token of its own
/// (one more than the last); a call of the engine naming an older one is
/// `ShareError::Stale`. Issuing a request drops the import of the one
/// before; the engine drops its pool. The engine's thread takes the
/// requests through [`SharedImage::link`] and provides textures, which
/// the UI thread imports, or publishes pictures drawn on the CPU.
///
/// When the owner is disposed, the link closes (`ShareLink::is_closed`),
/// its waits return at once and publishing does nothing; the import is
/// dropped, and the window signals the release fence past every frame.
#[track_caller]
pub fn shared_image(width: u32, height: u32) -> SharedImage {
    let at = Location::caller();
    let owner = kivact_runtime::Owner::current("shared_image", at);
    // Reading the adapter (below) asks for the window's device; the renewal
    // effect makes a GPU request once it is ready.
    let size = (width.max(1), height.max(1));
    let mode = mode_of(adapter());
    let state = Arc::new(Inbox::new(LinkState::new(ShareRequest {
        mode: mode.clone(),
        width: size.0,
        height: size.1,
        token: 1,
    })));
    let image = SharedImage {
        state: state.clone(),
        changed: owner.notifier(at),
        current: owner.signal(None, at),
        pixels: owner.signal(None, at),
        size: owner.signal(size, at),
    };
    // A new request when what the last one was made of changes, or its
    // import was lost.
    let mut sent = Sent::new();
    let _ = sent.update((mode, size));
    let issued = RefCell::new(sent);
    let renewing = image.clone();
    kivact_runtime::effect(move || {
        let mode = mode_of(adapter());
        let size = renewing.size.get();
        let lost = renewing.import_lost();
        let changed = issued.borrow_mut().update((mode.clone(), size)).is_some();
        if changed || lost {
            renewing.renew(mode, size);
        }
    });
    let receiving = image.clone();
    spawn_in(owner, receiving.receive(owner, at), at);
    owner.on_cleanup(move || {
        let closed = state.update(LinkState::close);
        drop(closed);
    });
    image
}

fn mode_of(adapter: Option<GpuAdapter>) -> ShareMode {
    adapter.map_or(ShareMode::Pixels, ShareMode::Gpu)
}

impl SharedImage {
    /// The engine's handle of the image (`Send + Sync + Clone`), to move
    /// to the engine's thread.
    pub fn link(&self) -> ShareLink {
        ShareLink::new(self.state.clone())
    }

    /// The newest frame the engine published for the import of the newest
    /// request, while it is `Ready`, to draw with `Draw::gpu_frame`;
    /// `None` otherwise. Tracked: a canvas that reads it redraws for each
    /// new frame.
    #[track_caller]
    pub fn frame(&self) -> Option<GpuFrame> {
        let current = self.current.try_get().flatten()?;
        if current.image.status() != ImportStatus::Ready {
            return None;
        }
        current.image.latest_frame()
    }

    /// The newest picture the engine published on the CPU
    /// (`ShareLink::publish_pixels`), whatever the request, to draw with
    /// `Draw::pixels`; `None` before the first. Tracked.
    #[track_caller]
    pub fn pixels(&self) -> Option<Pixels> {
        self.pixels.try_get().flatten()
    }

    /// Where the image stands ([`ShareStatus`]), tracked. `Closed` once
    /// its owner is gone.
    #[track_caller]
    pub fn status(&self) -> ShareStatus {
        self.changed.track();
        self.state.read(LinkState::status)
    }

    /// Ask the engine for pictures of `width` × `height` pixels (at least
    /// 1): a new request, unless the size is the one asked already.
    #[track_caller]
    pub fn set_size(&self, width: u32, height: u32) {
        self.size.set((width.max(1), height.max(1)));
    }

    /// Whether the import of the newest request was lost (a new device),
    /// tracked.
    fn import_lost(&self) -> bool {
        self.current
            .get()
            .is_some_and(|current| current.image.status() == ImportStatus::Lost)
    }

    /// Issue a request of `mode` and `size`, dropping the import of the
    /// one before.
    fn renew(&self, mode: ShareMode, size: (u32, u32)) {
        let dropped = self.state.update(|s| s.renew(mode, size));
        drop(dropped);
        if let Some(old) = self.replace_current(None)
            && Runtime::current().is_some()
        {
            old.owner.dispose();
        }
        self.changed.notify();
    }

    /// Point the image at `next` and return the import it replaces: the
    /// only writer of `current`. Writing `None` over `None` does nothing.
    fn replace_current(&self, next: Option<Current>) -> Option<Current> {
        let old = self.current.try_get_untracked().flatten();
        if old.is_some() || next.is_some() {
            self.current.set(next);
        }
        old
    }

    /// The task of the image: import what the engine provides, keep what
    /// it publishes on the CPU.
    async fn receive(self, owner: Owner, at: &'static Location<'static>) {
        loop {
            match self.state.next(LinkState::take_work).await {
                Work::Pixels(pixels) => self.pixels.set(Some(pixels)),
                Work::Import(token, textures) => self.import(token, textures, owner, at),
            }
        }
    }

    /// Import the textures of request `token` in a new child owner of
    /// `owner`, and follow its status.
    fn import(
        &self,
        token: u64,
        textures: SharedTextures,
        owner: Owner,
        at: &'static Location<'static>,
    ) {
        let Some(rt) = Runtime::current() else { return };
        let child = owner.child(at);
        let image = import_in(&rt, child, textures, at);
        self.replace_current(Some(Current {
            token,
            owner: child,
            image: image.clone(),
        }));
        self.changed.notify();
        let this = self.clone();
        child.with_setup(|| kivact_runtime::effect(move || this.follow(token, &image)));
    }

    /// Mirror the status of the import of request `token`.
    fn follow(&self, token: u64, image: &ExternalImage) {
        // A `Lost` import is renewed by the renewal effect; a `Dropped` one
        // has no effect left.
        let phase = match image.status() {
            ImportStatus::Pending | ImportStatus::Lost | ImportStatus::Dropped => return,
            ImportStatus::Ready => Phase::Ready(image.publisher()),
            ImportStatus::Failed(error) => Phase::Failed(error),
        };
        if self.state.update(|s| s.settle(token, phase)) {
            self.changed.notify();
        }
    }
}

impl fmt::Debug for SharedImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedImage")
            .field("status", &self.state.read(LinkState::status))
            .finish_non_exhaustive()
    }
}
