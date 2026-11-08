//! Role: The App's models, one per window, and the facts they share.
//!
//! The App owns one event queue, one asset store, one write accounting and
//! one motion clock; each window owns its own `Model`. A node's window is
//! derived from the model whose tree holds it (no second table).

use crate::contract::HostEvent;
use crate::lower::assets::ArvaneAssets;
use crate::model::Model;
use crate::model::events::{EventSink, Shared};
use crate::model::frame::state::FrameRequests;
use crate::mutation::Writes;
use kivact_types::{DrawCmd, MotionClock, NodeId, Size, WindowId, WindowRequest};
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

    /// The window whose tree draws `image` (its first such window).
    pub(crate) fn window_using_image(
        &self,
        image: kivact_types::ExternalImageId,
    ) -> Option<WindowId> {
        self.windows
            .iter()
            .find(|(_, model)| {
                model.tree.iter().any(|(_, node)| {
                    node.scenes().iter().flatten().any(|list| {
                        list.cmds.iter().any(|cmd| {
                            matches!(cmd, DrawCmd::External { frame, .. } if frame.image == image)
                        })
                    })
                })
            })
            .map(|(id, _)| *id)
    }

    /// Open a window and queue the host's open, sized `viewport`.
    pub(crate) fn open_window(&mut self, request: WindowRequest, viewport: Size) {
        let shared = Shared {
            events: self.events.clone(),
            assets: Rc::clone(&self.assets),
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

    /// Remove the model of `id`; the focus becomes none when it was there.
    pub(crate) fn remove_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
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
