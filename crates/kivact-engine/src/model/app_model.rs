//! Role: The App's models, one per window, and the facts they share.
//!
//! The App owns one event queue, one asset store, one write accounting and
//! one motion clock; each window owns its own `Model`. A node's window is
//! derived from the model whose tree holds it (no second table).

use crate::contract::{DevtoolsChange, HostEvent};
use crate::lower::assets::ArvaneAssets;
use crate::model::Model;
use crate::model::events::{EngineLog, EventSink, Shared};
use crate::model::frame::state::FrameRequests;
use crate::mutation::Writes;
use kivact_types::{ExternalImageId, MotionClock, NodeId, Size, WindowId, WindowRequest};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// A window change the host performs after the turn, in write order.
pub(crate) enum WindowChange {
    /// Open the requested window.
    Open(WindowRequest),
    /// Close the window.
    Close(WindowId),
}

/// Whether the App's motion clock runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AppClock {
    Running,
    /// No window can draw since `since_ns` (host ns).
    Stopped {
        since_ns: u64,
    },
}

/// Every window's model and the App-wide state.
pub(crate) struct AppModel {
    pub(crate) windows: BTreeMap<WindowId, Model>,
    pub(crate) events: EventSink,
    pub(crate) writes: Writes,
    pub(crate) assets: Rc<RefCell<ArvaneAssets>>,
    /// Window changes the host performs after the turn, in order.
    pub(crate) window_changes: Vec<WindowChange>,
    /// The App's one motion clock; `FrameTick.clock` carries it.
    pub(crate) motion: MotionClock,
    /// Whether the App's motion clock runs: it stops while no window can draw.
    pub(crate) clock: AppClock,
    /// The frames the core asked for; only the tick-source window's
    /// `Model::tick` reads them.
    pub(crate) client_frames: FrameRequests,
    /// The window the OS gives the keyboard focus (`WindowInfo.focused`).
    pub(crate) focused_window: Option<WindowId>,
    /// The window each imported external image was imported into, until it is dropped.
    pub(crate) external_windows: BTreeMap<ExternalImageId, WindowId>,
    /// The DevTools changes the host applies after the turn, in write order.
    pub(crate) devtools_changes: Vec<DevtoolsChange>,
    /// What the DevTools records, shared by every model and the host.
    pub(crate) log: EngineLog,
}

impl AppModel {
    /// The App of the first window `main`.
    pub(crate) fn new(shared: Shared, main: Model) -> Self {
        let window = main.window;
        let mut windows = BTreeMap::new();
        windows.insert(window, main);
        Self {
            windows,
            events: shared.events,
            writes: Writes::default(),
            assets: shared.assets,
            window_changes: Vec::new(),
            motion: MotionClock::default(),
            clock: AppClock::Running,
            client_frames: FrameRequests::default(),
            focused_window: None,
            external_windows: BTreeMap::new(),
            devtools_changes: Vec::new(),
            log: shared.log,
        }
    }

    /// The model of `id`.
    pub(crate) fn model(&self, id: WindowId) -> Option<&Model> {
        self.windows.get(&id)
    }

    /// The model of `id`, mutably.
    pub(crate) fn model_mut(&mut self, id: WindowId) -> Option<&mut Model> {
        self.windows.get_mut(&id)
    }

    /// The window whose model holds `node`.
    pub(crate) fn window_of(&self, node: NodeId) -> Option<WindowId> {
        self.windows
            .iter()
            .find(|(_, model)| model.tree.get(node).is_some())
            .map(|(id, _)| *id)
    }

    /// Run `f` with the model of `node`'s window and the App's writes.
    pub(crate) fn with_node<R>(
        &mut self,
        node: NodeId,
        f: impl FnOnce(&mut Model, &mut Writes) -> R,
    ) -> Option<R> {
        let window = self.window_of(node)?;
        let Self {
            windows, writes, ..
        } = self;
        let model = windows.get_mut(&window)?;
        Some(f(model, writes))
    }

    /// Open a window and queue the host's open, sized `viewport`.
    pub(crate) fn open_window(&mut self, request: WindowRequest, viewport: Size) {
        let shared = Shared {
            events: self.events.clone(),
            assets: Rc::clone(&self.assets),
            log: self.log.clone(),
        };
        let model = Model::new(request.id, viewport, &shared);
        self.windows.insert(request.id, model);
        self.window_changes.push(WindowChange::Open(request));
    }

    /// Queue the host's close of `id`.
    pub(crate) fn close_window(&mut self, id: WindowId) {
        self.window_changes.push(WindowChange::Close(id));
    }

    /// The window changes since the last call, in order.
    pub(crate) fn take_window_changes(&mut self) -> Vec<WindowChange> {
        std::mem::take(&mut self.window_changes)
    }

    /// The DevTools changes since the last call, in write order.
    pub(crate) fn take_devtools_changes(&mut self) -> Vec<DevtoolsChange> {
        std::mem::take(&mut self.devtools_changes)
    }

    /// Queue a DevTools change the host applies after the turn.
    pub(crate) fn push_devtools_change(&mut self, change: DevtoolsChange) {
        self.devtools_changes.push(change);
    }

    /// Remove the model of `id`; the focus becomes none when it was there,
    /// and the images imported into it are no longer known.
    pub(crate) fn remove_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
        self.external_windows.retain(|_, window| *window != id);
        if self.focused_window == Some(id) {
            self.focused_window = None;
        }
    }

    /// The open windows, ascending by id.
    pub(crate) fn windows(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }

    /// The App's event queue.
    pub(crate) fn events(&self) -> &EventSink {
        &self.events
    }

    /// The App's write accounting.
    pub(crate) fn writes(&self) -> &Writes {
        &self.writes
    }

    /// The App's write accounting, mutably.
    pub(crate) fn writes_mut(&mut self) -> &mut Writes {
        &mut self.writes
    }

    /// The App's one motion clock.
    pub(crate) fn motion(&self) -> MotionClock {
        self.motion
    }

    /// The window the OS focuses.
    pub(crate) fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Queue `event` after every event queued so far.
    pub(crate) fn push_event(&self, event: HostEvent) {
        self.events.push(event);
    }
}
