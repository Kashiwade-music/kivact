//! Role: Images an external render engine shares, imported into the window's GPU device.
//!
//! The implementation of `kivact::gpu_interop` (feature `gpu-interop`):
//! the adapter as a tracked environment value, the imports an App asked
//! for, their status, and the frames a canvas draws.
//!
//! Invariants:
//! - The App names each import with an id of its own (`ExternalImageId`,
//!   counted from 1 per App, never reused); an image belongs to the App's
//!   one window.
//! - An import is one `Writer::import_external`, carried out by the host
//!   after the turn and answered by `HostEvent::ExternalImported`; the
//!   cleanup of the owner it was called in writes `Writer::drop_external`,
//!   whatever handles to it are left, and forgets it (a later answer is
//!   ignored).
//! - The adapter signal lives in the App's [`State`] (a runtime extension
//!   the client installs; the root owner makes the signal before the root
//!   is mounted). It starts `None` and follows
//!   `HostEvent::GpuAdapterChanged`. An import that is `Ready` is `Lost`
//!   once the adapter's generation is not the one it was imported into
//!   (a new device, or none).
//! - Each import has one `FramePublisher` state ([`inbox`]) and a task of
//!   its owner that moves the newest published frame into the tracked
//!   `latest` signal; the owner's cleanup closes the state, so publishing
//!   after it does nothing.

pub(crate) mod inbox;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;

use kivact_reactive::Signal;
use kivact_reactive::{ErrorKind, UiError};

use kivact_reactive::{Owner, Runtime, require_effectful};
use kivact_wire::{ExternalFrame, ExternalImageId, ExternalImport, GpuAdapter, SharedTextures};

use crate::view::state::ViewState;
use inbox::Inbox;
use kivact_style::draw::GpuFrame;
use kivact_task::spawn_in;

/// What `gpu_interop` keeps per App: the imports and the adapter signal.
/// The client installs it as a runtime extension when the App is created
/// (`install_adapter` runs once the root owner exists).
#[derive(Default)]
pub(crate) struct State {
    imports: RefCell<Imports>,
    /// The adapter of the window's GPU device that can import shared
    /// textures (`None` until the host reports one); the signal is made
    /// by [`State::install_adapter`].
    adapter: Cell<Option<Signal<Option<GpuAdapter>>>>,
}

impl State {
    /// The gpu-interop state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &State {
        rt.ext::<State>()
    }

    /// Make the adapter signal in the root owner `owner`, created at `at`.
    pub(crate) fn install_adapter(&self, owner: Owner, at: &'static Location<'static>) {
        self.adapter.set(Some(owner.signal(None::<GpuAdapter>, at)));
    }
}

/// The imports of an App, by id.
#[derive(Default)]
struct Imports {
    /// The last id given.
    last: u64,
    images: HashMap<ExternalImageId, Import>,
}

/// One import the App asked for and has not dropped.
struct Import {
    status: Signal<ImportStatus>,
    /// The generation of the device it was imported into, while `Ready`.
    generation: Option<u64>,
    /// Where `import` was called: its failure is located there.
    at: &'static Location<'static>,
}

/// Where an import ([`import`]) stands.
///
/// It is `Pending` from the call until the window answers, after the turn
/// that called it. Then:
/// - `Ready`: the textures are in the window's device; draw their frames
///   ([`ExternalImage::frame`]).
/// - `Failed`: the window could not import them. An error of kind
///   `ErrorKind::Unsupported` when the window has no GPU device that can
///   import (it draws on the CPU, its device is not ready, or the platform
///   is neither Windows nor Linux); of kind `ErrorKind::Platform` when the
///   textures do not fit what they say or the window's device (the
///   adapter, the graphics API, the size, handles it refuses) or the
///   device failed, with Arvane's message. The error is located at the
///   call.
/// - `Lost`: a `Ready` import whose device is gone: the adapter
///   ([`adapter`]) has a new [`GpuAdapter::generation`] or is `None`.
///   Make new textures on the new adapter and import them again. Also what
///   [`ExternalImage::status`] reads once the import's owner is gone.
#[derive(Clone, Debug, PartialEq)]
pub enum ImportStatus {
    /// Not answered yet.
    Pending,
    /// Imported: its frames draw.
    Ready,
    /// Refused (see the type's doc for the kinds).
    Failed(UiError),
    /// Its device is gone: import again.
    Lost,
}

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
    status: Signal<ImportStatus>,
    /// The size in pixels of the imported textures.
    size: (u32, u32),
    /// The newest frame a [`FramePublisher`] published.
    latest: Signal<Option<GpuFrame>>,
    published: Arc<Inbox<Published>>,
}

impl ExternalImage {
    /// Where the import stands ([`ImportStatus`]), tracked: a view or
    /// effect that reads it follows its changes. `Lost` once its owner is
    /// gone.
    #[track_caller]
    pub fn status(&self) -> ImportStatus {
        self.status.try_get().unwrap_or(ImportStatus::Lost)
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
#[derive(Default)]
struct Published {
    /// The import's owner is gone.
    closed: bool,
    /// The newest `(slot, value)` the UI has not taken yet.
    newest: Option<(u32, u64)>,
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
            if !p.closed {
                p.newest = Some((slot, value));
            }
        });
    }

    /// Whether the import's owner is gone: the image was dropped, and
    /// the engine may stop drawing for it.
    pub fn is_closed(&self) -> bool {
        self.published.read(|p| p.closed)
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

/// The adapter of the GPU device drawing the App's window, when the
/// device can import shared textures (`None` until it is ready, while the
/// window draws on the CPU, and on platforms other than Windows and
/// Linux), tracked: a view or effect that reads it follows its changes.
/// Make the engine's device on this adapter, with its graphics API
/// ([`GpuAdapter::backend`]), then [`import`] its textures. A new
/// [`GpuAdapter::generation`] means a new device: every import before it
/// is `Lost`.
///
/// Like `platform::window_size`, it resolves the nearest owner at the
/// call (setup, handler, task or effect); a located `SetupOnly` misuse
/// outside every owner.
#[track_caller]
pub fn adapter() -> Option<GpuAdapter> {
    let rt = kivact_reactive::Owner::ambient("adapter", Location::caller()).runtime();
    let adapter = State::of(&rt).adapter.get();
    drop(rt);
    adapter.and_then(|adapter| adapter.get())
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
/// Import when [`adapter`] is `Some`, with textures made on that adapter.
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
    let owner = kivact_reactive::Owner::ambient("import", at);
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
    let status: Signal<ImportStatus> = owner.signal(ImportStatus::Pending, at);
    let id = {
        let mut imports = State::of(rt).imports.borrow_mut();
        imports.last += 1;
        let id = ExternalImageId(imports.last);
        imports.images.insert(
            id,
            Import {
                status,
                generation: None,
                at,
            },
        );
        id
    };
    ViewState::tree(rt).engine().import_external(id, textures);
    rt.request_turn();
    let latest: Signal<Option<GpuFrame>> = owner.signal(None::<GpuFrame>, at);
    let published = Arc::new(Inbox::new(Published::default()));
    let inbox = published.clone();
    spawn_in(
        owner,
        async move {
            loop {
                let (slot, value) = inbox.next(|p| p.newest.take()).await;
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
        closing.update(|p| {
            p.closed = true;
            p.newest = None;
        });
        let Some(rt) = weak.upgrade() else { return };
        State::of(&rt).imports.borrow_mut().images.remove(&id);
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

/// The window answered the import `image` (`HostEvent::ExternalImported`).
pub(crate) fn imported(rt: &Runtime, image: ExternalImageId, result: &ExternalImport) {
    let mut imports = State::of(rt).imports.borrow_mut();
    // Dropped meanwhile (its owner is gone).
    let Some(import) = imports.images.get_mut(&image) else {
        return;
    };
    let failed = |kind: ErrorKind, message: &str| {
        ImportStatus::Failed(UiError::new(
            kind,
            format!("gpu_interop::import: {message}"),
            Some(import.at),
        ))
    };
    let status = match result {
        ExternalImport::Ready { generation } => {
            import.generation = Some(*generation);
            ImportStatus::Ready
        }
        ExternalImport::Unavailable(message) => failed(ErrorKind::Unsupported, message),
        ExternalImport::Failed(message) => failed(ErrorKind::Platform, message),
    };
    let signal = import.status;
    drop(imports);
    // Only the import's status is written here: no owner applies.
    rt.enter(|| {
        signal.set(status);
    })
}

/// The window's adapter changed (`HostEvent::GpuAdapterChanged`): the
/// imports of another device are lost.
pub(crate) fn adapter_changed(rt: &Runtime, adapter: Option<&GpuAdapter>) {
    let generation = adapter.map(|a| a.generation);
    let state = State::of(rt);
    let lost: Vec<Signal<ImportStatus>> = state
        .imports
        .borrow_mut()
        .images
        .values_mut()
        .filter(|import| import.generation.is_some_and(|g| Some(g) != generation))
        .map(|import| {
            import.generation = None;
            import.status
        })
        .collect();
    let signal = state.adapter.get();
    // Only the adapter and the statuses are written here: no owner applies.
    rt.enter(|| {
        if let Some(signal) = signal {
            signal.set(adapter.cloned());
        }
        for status in lost {
            status.set(ImportStatus::Lost);
        }
    })
}
