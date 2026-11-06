//! Role: Import an external image: its handle, status, frames and frame publisher.

use super::inbox::Inbox;
use super::registry::{ImportPhase, ImportStatus, State};
use crate::view::state::ViewState;
use kivact_runtime::Signal;
use kivact_runtime::spawn_in;
use kivact_runtime::{Owner, Runtime, require_effectful};
use kivact_style::draw::GpuFrame;
use kivact_types::{ExternalFrame, ExternalImageId, SharedTextures};
use std::fmt;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;

/// An image imported into the App's window ([`import`]): a handle of the
/// UI thread, `Clone` and `!Send`.
///
/// Unlike `Pixels`, whose buffer lives while a value holds it, the
/// imported textures belong to the owner [`import`] was called in: when
/// that owner is disposed (its `show` turns false, its row goes, the App
/// closes), the window drops the image, even while clones of this handle
/// are left. Its frames then draw nothing, and the window signals the
/// release fence past every frame of it.
#[derive(Clone)]
pub struct ExternalImage {
    id: ExternalImageId,
    status: Signal<ImportPhase>,
    /// The size in pixels of the imported textures.
    size: (u32, u32),
    /// The newest frame a [`FramePublisher`] published.
    latest: Signal<Option<GpuFrame>>,
    published: Arc<Inbox<Published>>,
}

impl ExternalImage {
    /// Where the import stands ([`ImportStatus`]), tracked: a view or
    /// effect that reads it follows its changes. `Dropped` once its owner
    /// is gone.
    #[track_caller]
    pub fn status(&self) -> ImportStatus {
        self.status
            .try_get()
            .map_or(ImportStatus::Dropped, |phase| phase.status())
    }

    /// The frame the engine drew into the pool's texture `slot` (an index
    /// into `SharedTextures::textures`) and signaled ready with the value
    /// `value`, to draw with `Draw::gpu_frame`. The frame has the size of
    /// the imported textures ([`GpuFrame::width`], [`GpuFrame::height`]).
    pub fn frame(&self, slot: u32, value: u64) -> GpuFrame {
        GpuFrame::new(
            ExternalFrame {
                image: self.id,
                slot,
                value,
            },
            self.size.0,
            self.size.1,
        )
    }

    /// A handle for the engine's thread (`Send + Sync + Clone`) that
    /// publishes the image's frames ([`FramePublisher::publish`]) to
    /// [`ExternalImage::latest_frame`], without a channel and a task of the
    /// App's own.
    pub fn publisher(&self) -> FramePublisher {
        FramePublisher {
            published: self.published.clone(),
        }
    }

    /// The newest frame published through [`ExternalImage::publisher`]
    /// (as [`ExternalImage::frame`] makes it), tracked: a canvas that
    /// reads it redraws for each new frame. `None` until the first one,
    /// and once the import's owner is gone.
    #[track_caller]
    pub fn latest_frame(&self) -> Option<GpuFrame> {
        self.latest.try_get().flatten()
    }
}

/// What the engine's threads publish for an import.
enum Published {
    /// The import's owner lives; the newest `(slot, value)` the UI has not
    /// taken yet.
    Open { newest: Option<(u32, u64)> },
    /// The import's owner is gone.
    Closed,
}

/// Publishes the frames of an [`ExternalImage`] from the engine's thread
/// ([`ExternalImage::publisher`]): `Send + Sync + Clone`.
///
/// Each [`FramePublisher::publish`] wakes the UI thread, which sets the
/// image's [`ExternalImage::latest_frame`]; a canvas reading it redraws.
/// The newest frame wins: frames published before the UI thread took the
/// last one are skipped. Skipping is safe for the engine's pool: Arvane
/// signals the release fence to the value it drew, less one, so the
/// frames skipped before a drawn one are released with it (the contract
/// of `SharedTextures`).
///
/// Once the import's owner is gone the publisher is closed
/// ([`FramePublisher::is_closed`]) and publishing does nothing.
#[derive(Clone)]
pub struct FramePublisher {
    published: Arc<Inbox<Published>>,
}

impl FramePublisher {
    /// Publish the frame the engine drew into the pool's texture `slot`
    /// and signaled ready with `value` (as [`ExternalImage::frame`] names
    /// it), from any thread. Nothing once closed.
    pub fn publish(&self, slot: u32, value: u64) {
        self.published.update(|p| {
            if let Published::Open { newest } = p {
                *newest = Some((slot, value));
            }
        });
    }

    /// Whether the import's owner is gone: the image was dropped, and
    /// the engine may stop drawing for it.
    pub fn is_closed(&self) -> bool {
        self.published.read(|p| matches!(p, Published::Closed))
    }
}

impl fmt::Debug for FramePublisher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramePublisher")
            .field("closed", &self.is_closed())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for ExternalImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExternalImage")
            .field("id", &self.id.0)
            .finish_non_exhaustive()
    }
}

/// Import the pool of textures an external render engine shares into the
/// device of the App's window, as one image; draw its frames with
/// [`ExternalImage::frame`] and `Draw::gpu_frame`.
///
/// The import belongs to the nearest owner at the call (setup, handler,
/// task or effect; a located `SetupOnly` misuse outside every owner, and
/// a misuse in a memo or a render effect): disposing the owner drops it.
/// The window imports the textures after the turn; the answer arrives
/// as the status ([`ImportStatus`]), never as an error of the owner.
/// Import when [`adapter`](super::adapter) is `Some`, with textures made on that adapter.
///
/// # Safety
///
/// The engine keeps the contract of `SharedTextures` (its rustdoc, "The
/// engine's contract"): how the textures and fences are made, the
/// hand-off state, writing a texture only after the import is `Ready` and
/// only once the release fence reached the value of its last frame, and
/// new textures after a new device. Breaking it is undefined behavior of
/// the graphics API (Vulkan and Direct3D 12): a broken picture, a lost
/// device, a crashed process, or access to GPU memory outside the resource
/// are all possible. Kivact and Arvane cannot check most of it, which is
/// why this is an `unsafe fn`.
#[track_caller]
#[allow(
    unsafe_code,
    reason = "the engine's side of GPU texture sharing is a contract the caller keeps"
)]
pub unsafe fn import(textures: SharedTextures) -> ExternalImage {
    let at = Location::caller();
    let owner = kivact_runtime::Owner::ambient("import", at);
    let rt = owner.runtime();
    require_effectful("import");
    import_in(&rt, owner, textures, at)
}

/// Import `textures` as an image of `owner` ([`import`], whose contract
/// the caller keeps), located at `at`.
pub(crate) fn import_in(
    rt: &Rc<Runtime>,
    owner: Owner,
    textures: SharedTextures,
    at: &'static Location<'static>,
) -> ExternalImage {
    let size = (textures.width, textures.height);
    let status: Signal<ImportPhase> = owner.signal(ImportPhase::Pending, at);
    let id = State::of(rt).register(status, at);
    ViewState::tree(rt).engine().import_external(id, textures);
    rt.request_turn();
    let latest: Signal<Option<GpuFrame>> = owner.signal(None::<GpuFrame>, at);
    let published = Arc::new(Inbox::new(Published::Open { newest: None }));
    let inbox = published.clone();
    spawn_in(
        owner,
        async move {
            loop {
                let (slot, value) = inbox
                    .next(|p| match p {
                        Published::Open { newest } => newest.take(),
                        Published::Closed => None,
                    })
                    .await;
                latest.set(Some(GpuFrame::new(
                    ExternalFrame {
                        image: id,
                        slot,
                        value,
                    },
                    size.0,
                    size.1,
                )));
            }
        },
        at,
    );
    let weak = Rc::downgrade(rt);
    let closing = published.clone();
    owner.on_cleanup(move || {
        closing.update(|p| *p = Published::Closed);
        let Some(rt) = weak.upgrade() else { return };
        State::of(&rt).forget(id);
        ViewState::tree(&rt).engine().drop_external(id);
    });
    ExternalImage {
        id,
        status,
        size,
        latest,
        published,
    }
}
