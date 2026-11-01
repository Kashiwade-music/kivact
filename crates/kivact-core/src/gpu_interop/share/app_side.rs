//! Role: The App's side of a shared image: status, size, renewal and the import it follows.

use std::cell::RefCell;
use std::fmt;
use std::panic::Location;
use std::sync::Arc;

use kivact_reactive::Signal;
use kivact_reactive::UiError;
use kivact_reactive::{Owner, Runtime};
use kivact_wire::{GpuAdapter, SharedTextures};

use super::link::{LinkState, Phase, ShareLink, ShareMode, ShareRequest};
use crate::gpu_interop::import::inbox::Inbox;
use crate::gpu_interop::import::{ExternalImage, ImportStatus, adapter, import_in};
use kivact_style::draw::{GpuFrame, Pixels};
use kivact_task::spawn_in;

/// Where a [`SharedImage`] stands, tracked ([`SharedImage::status`]).
#[derive(Clone, Debug, PartialEq)]
pub enum ShareStatus {
    /// A [`ShareMode::Gpu`] request was issued; the engine has not
    /// provided textures for it yet.
    Waiting,
    /// The engine provided textures; the window is importing them.
    Importing,
    /// The textures are imported: the engine's frames draw
    /// ([`SharedImage::frame`]).
    Ready,
    /// The request is [`ShareMode::Pixels`]: the window cannot import, and
    /// the engine publishes pictures drawn on the CPU
    /// ([`SharedImage::pixels`]).
    Pixels,
    /// The window could not import the textures (the kinds of
    /// `ImportStatus::Failed`). The image stays so until the next request.
    Failed(UiError),
}

/// An image an engine on another thread shares with the UI
/// ([`shared_image`]): the UI thread's handle, `Clone` and `!Send`.
///
/// Hand the engine its [`SharedImage::link`]; draw [`SharedImage::frame`]
/// with `Draw::gpu_frame`, and [`SharedImage::pixels`] with `Draw::pixels`
/// when there is no frame.
#[derive(Clone)]
pub struct SharedImage {
    state: Arc<Inbox<LinkState>>,
    status: Signal<ShareStatus>,
    /// The import of the newest request, once the engine provided it.
    current: Signal<Option<Current>>,
    pixels: Signal<Option<Pixels>>,
    size: Signal<(u32, u32)>,
    /// Counts the imports found `Lost`.
    lost: Signal<u64>,
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

/// What the engine handed the UI.
enum Work {
    Import(u64, SharedTextures),
    Pixels(Pixels),
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
    let owner = kivact_reactive::Owner::current("shared_image", at);
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
        status: owner.signal(status_of(&mode), at),
        current: owner.signal(None, at),
        pixels: owner.signal(None, at),
        size: owner.signal(size, at),
        lost: owner.signal(0, at),
    };
    // A new request when what the last one was made of changes.
    let issued = RefCell::new((mode, size, 0));
    let renewing = image.clone();
    kivact_reactive::effect(move || {
        let wanted = (mode_of(adapter()), renewing.size.get(), renewing.lost.get());
        if *issued.borrow() == wanted {
            return;
        }
        let (mode, size, _) = wanted.clone();
        *issued.borrow_mut() = wanted;
        renewing.renew(mode, size);
    });
    let receiving = image.clone();
    spawn_in(owner, receiving.receive(owner, at), at);
    owner.on_cleanup(move || {
        let dropped = state.update(|s| {
            s.closed = true;
            s.publisher = None;
            s.pixels = None;
            s.provided.take()
        });
        drop(dropped);
    });
    image
}

fn mode_of(adapter: Option<GpuAdapter>) -> ShareMode {
    adapter.map_or(ShareMode::Pixels, ShareMode::Gpu)
}

fn status_of(mode: &ShareMode) -> ShareStatus {
    match mode {
        ShareMode::Gpu(_) => ShareStatus::Waiting,
        ShareMode::Pixels => ShareStatus::Pixels,
    }
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

    /// Where the image stands ([`ShareStatus`]), tracked. `Waiting` once
    /// its owner is gone.
    #[track_caller]
    pub fn status(&self) -> ShareStatus {
        self.status.try_get().unwrap_or(ShareStatus::Waiting)
    }

    /// Ask the engine for pictures of `width` × `height` pixels (at least
    /// 1): a new request, unless the size is the one asked already.
    #[track_caller]
    pub fn set_size(&self, width: u32, height: u32) {
        self.size.set((width.max(1), height.max(1)));
    }

    /// Issue a request of `mode` and `size`, dropping the import of the
    /// one before.
    fn renew(&self, mode: ShareMode, size: (u32, u32)) {
        let status = status_of(&mode);
        let dropped = self.state.update(|s| s.renew(mode, size));
        drop(dropped);
        if let Some(old) = self.current.try_get_untracked().flatten() {
            self.current.set(None);
            if Runtime::current().is_some() {
                old.owner.dispose();
            }
        }
        self.status.set(status);
    }

    /// The task of the image: import what the engine provides, keep what
    /// it publishes on the CPU.
    async fn receive(self, owner: Owner, at: &'static Location<'static>) {
        loop {
            let work = self
                .state
                .next(|s| match s.provided.take() {
                    Some((token, textures)) => Some(Work::Import(token, textures)),
                    None => s.pixels.take().map(Work::Pixels),
                })
                .await;
            match work {
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
        self.current.set(Some(Current {
            token,
            owner: child,
            image: image.clone(),
        }));
        self.status.set(ShareStatus::Importing);
        let this = self.clone();
        child.with_setup(|| kivact_reactive::effect(move || this.follow(token, &image)));
    }

    /// Mirror the status of the import of request `token`.
    fn follow(&self, token: u64, image: &ExternalImage) {
        let (phase, status) = match image.status() {
            ImportStatus::Pending => return,
            ImportStatus::Ready => (Phase::Ready, ShareStatus::Ready),
            ImportStatus::Failed(error) => (
                Phase::Failed(error.message().to_owned()),
                ShareStatus::Failed(error),
            ),
            ImportStatus::Lost => {
                self.lost
                    .set(self.lost.try_get_untracked().unwrap_or(0) + 1);
                return;
            }
        };
        let publisher = (phase == Phase::Ready).then(|| image.publisher());
        let current = self.state.update(|s| {
            let current = s.request.token == token && !s.closed;
            if current {
                s.phase = phase;
                s.publisher = publisher;
            }
            current
        });
        if current {
            self.status.set(status);
        }
    }
}

impl fmt::Debug for SharedImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedImage")
            .field("request", &self.state.read(|s| s.request.clone()))
            .finish_non_exhaustive()
    }
}
