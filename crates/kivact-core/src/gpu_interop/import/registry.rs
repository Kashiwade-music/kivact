//! Role: The App's record of its imports and the adapter signal.

use kivact_reactive::Signal;
use kivact_reactive::{ErrorKind, UiError};
use kivact_reactive::{Owner, Runtime};
use kivact_wire::{ExternalImageId, ExternalImport, GpuAdapter};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::panic::Location;

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

impl State {
    /// Record a new import with `status` and give it its id.
    pub(super) fn register(
        &self,
        status: Signal<ImportStatus>,
        at: &'static Location<'static>,
    ) -> ExternalImageId {
        let mut imports = self.imports.borrow_mut();
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
    }

    /// Forget the import `id` (its owner is gone).
    pub(super) fn forget(&self, id: ExternalImageId) {
        self.imports.borrow_mut().images.remove(&id);
    }
}

/// Where an import ([`import`](super::import)) stands.
///
/// It is `Pending` from the call until the window answers, after the turn
/// that called it. Then:
/// - `Ready`: the textures are in the window's device; draw their frames
///   ([`ExternalImage::frame`](super::ExternalImage::frame)).
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
///   [`ExternalImage::status`](super::ExternalImage::status) reads once the import's owner is gone.
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

/// The adapter of the GPU device drawing the App's window, when the
/// device can import shared textures (`None` until it is ready, while the
/// window draws on the CPU, and on platforms other than Windows and
/// Linux), tracked: a view or effect that reads it follows its changes.
/// Make the engine's device on this adapter, with its graphics API
/// ([`GpuAdapter::backend`]), then [`import`](super::import) its textures. A new
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
