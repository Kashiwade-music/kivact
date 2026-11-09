//! Role: The App's record of its imports and the adapters of its windows.

use crate::view::state::current_window;
use kivact_runtime::Signal;
use kivact_runtime::{ErrorKind, UiError};
use kivact_runtime::{Owner, Runtime};
use kivact_types::{ExternalImageId, ExternalImport, GpuAdapter, WindowId};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::panic::Location;

/// What `gpu_interop` keeps per App: its imports and the adapter of each
/// window's device. The client installs it as a runtime extension when the
/// App is created.
pub(crate) struct State {
    imports: RefCell<Imports>,
    /// The adapter of the device drawing each window; a window without an
    /// entry draws on the CPU or has no device yet.
    adapter: Signal<BTreeMap<WindowId, GpuAdapter>>,
}

impl State {
    pub(crate) fn new(owner: Owner, at: &'static Location<'static>) -> Self {
        Self {
            imports: RefCell::default(),
            adapter: owner.signal(BTreeMap::new(), at),
        }
    }

    /// The gpu-interop state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &State {
        rt.ext::<State>()
    }
}

/// The imports of an App, by id.
#[derive(Default)]
struct Imports {
    /// The last id given.
    last: u64,
    images: HashMap<ExternalImageId, Import>,
}

/// Where an import stands inside the App: the public [`ImportStatus`],
/// with the device generation of a `Ready` one.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ImportPhase {
    Pending,
    /// Imported into the device of this `generation`.
    Ready {
        generation: u64,
    },
    Failed(UiError),
    Lost,
}

impl ImportPhase {
    /// The status the App reads.
    pub(super) fn status(&self) -> ImportStatus {
        match self {
            ImportPhase::Pending => ImportStatus::Pending,
            ImportPhase::Ready { .. } => ImportStatus::Ready,
            ImportPhase::Failed(error) => ImportStatus::Failed(error.clone()),
            ImportPhase::Lost => ImportStatus::Lost,
        }
    }
}

/// One import the App asked for and has not dropped.
struct Import {
    /// The window whose device imports the textures.
    window: WindowId,
    status: Signal<ImportPhase>,
    /// Where `import` was called: its failure is located there.
    at: &'static Location<'static>,
}

impl State {
    /// Record a new import into `window`'s device with `status` and give it its id.
    pub(super) fn register(
        &self,
        window: WindowId,
        status: Signal<ImportPhase>,
        at: &'static Location<'static>,
    ) -> ExternalImageId {
        let mut imports = self.imports.borrow_mut();
        imports.last += 1;
        let id = ExternalImageId(imports.last);
        imports.images.insert(id, Import { window, status, at });
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
///   Make new textures on the new adapter and import them again.
/// - `Dropped`: the owner [`import`](super::import) was called in is gone;
///   the window dropped the image and signals its release fence past every
///   frame.
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
    /// The import's owner is gone: the window dropped the image.
    Dropped,
}

/// The adapter of the GPU device drawing the window the call belongs to
/// (`None` until its device is ready, while the window draws on the CPU,
/// and on platforms other than Windows and Linux), tracked: a view or
/// effect that reads it follows its changes. Make the engine's device on
/// this adapter, with its graphics API ([`GpuAdapter::backend`]), then
/// [`import`](super::import) its textures. A new [`GpuAdapter::generation`]
/// means a new device: every import of the window before it is `Lost`.
///
/// Like `platform::window_size`, it resolves the nearest owner at the
/// call (setup, handler, task or effect); a located `SetupOnly` misuse
/// outside every owner.
#[track_caller]
pub fn adapter() -> Option<GpuAdapter> {
    let owner = Owner::ambient("adapter", Location::caller());
    let window = current_window(owner);
    let rt = owner.runtime();
    let signal = State::of(&rt).adapter;
    drop(rt);
    signal.with(|adapters| adapters.get(&window).cloned())
}

/// The window answered the import `image` (`HostEvent::ExternalImported`).
pub(crate) fn imported(rt: &Runtime, image: ExternalImageId, result: &ExternalImport) {
    let imports = State::of(rt).imports.borrow();
    // Dropped meanwhile (its owner is gone).
    let Some(import) = imports.images.get(&image) else {
        return;
    };
    let failed = |kind: ErrorKind, message: &str| {
        ImportPhase::Failed(UiError::new(
            kind,
            format!("gpu_interop::import: {message}"),
            Some(import.at),
        ))
    };
    let status = match result {
        ExternalImport::Ready { generation } => ImportPhase::Ready {
            generation: *generation,
        },
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

/// The adapter of `window`'s device changed (`HostEvent::GpuAdapterChanged`):
/// its imports of another device are lost. The other windows keep theirs.
pub(crate) fn adapter_changed(rt: &Runtime, window: WindowId, adapter: Option<&GpuAdapter>) {
    let generation = adapter.map(|a| a.generation);
    let statuses: Vec<Signal<ImportPhase>> = State::of(rt)
        .imports
        .borrow()
        .images
        .values()
        .filter(|import| import.window == window)
        .map(|import| import.status)
        .collect();
    let signal = State::of(rt).adapter;
    // Only the adapter and the statuses are written here: no owner applies.
    rt.enter(|| {
        let mut adapters = signal.get_untracked();
        match adapter {
            Some(adapter) => adapters.insert(window, adapter.clone()),
            None => adapters.remove(&window),
        };
        signal.set(adapters);
        for status in statuses {
            let lost = status.with_untracked(|phase| {
                matches!(phase, ImportPhase::Ready { generation: g } if Some(*g) != generation)
            });
            if lost {
                status.set(ImportPhase::Lost);
            }
        }
    })
}
