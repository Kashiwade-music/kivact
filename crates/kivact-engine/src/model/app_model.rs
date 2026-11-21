//! Role: The App's models, one per window, and the facts they share.
//!
//! The App owns one event queue, one asset store, one write accounting and
//! one motion clock; each window owns its own `Model`. A node's window is
//! derived from the model whose tree holds it (no second table).

use crate::contract::{DevtoolsChange, HostEvent, HostRequest, NodeFlags};
use crate::lower::assets::ArvaneAssets;
use crate::model::Model;
use crate::model::events::{EngineLog, EventSink, Shared};
use crate::model::frame::state::FrameRequests;
use crate::mutation::Writes;
use kivact_types::{ExternalImageId, NodeId, RequestId, Size, WindowId, WindowRequest};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

/// A window change the host performs after the turn, in write order.
pub(crate) enum WindowChange {
    /// Open the requested window.
    Open(WindowRequest),
    /// Close the window.
    Close(WindowId),
}

/// One open window: its model and the host's state of its platform window.
pub(crate) struct Window {
    pub(crate) model: Model,
    pub(crate) host: super::window::HostWindow,
}

// xtask-allow(state): legacy: assets, focused_window
/// Every window's model and the App-wide state.
pub(crate) struct AppModel {
    /// Every window: its model from the write that opens it, and its host state (not drawable until the platform window opens).
    pub(crate) windows: BTreeMap<WindowId, Window>,
    pub(crate) events: EventSink,
    pub(crate) writes: Writes,
    pub(crate) assets: Rc<RefCell<ArvaneAssets>>,
    /// Window changes the host performs after the turn, in order.
    pub(crate) window_changes: Vec<WindowChange>,
    /// The platform and App-wide requests the host carries out after the
    /// turn, in write order (`Host::effects`).
    pub(crate) effects: Vec<(RequestId, HostRequest)>,
    /// The App's one motion clock (`kivact_types::clock::Clock`): it stops while no window can draw; `FrameTick.clock` carries its snapshot.
    pub(crate) clock: kivact_types::clock::Clock,
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
    /// The injections queued for each window, oldest first: the shell
    /// events of a window come from the first until the shell reports it
    /// delivered (`ShellEvent::Injected`).
    pub(crate) injecting: BTreeMap<WindowId, std::collections::VecDeque<RequestId>>,
    /// The `TextChanged` events of each node handed to the client in the
    /// running turn and not yet dispatched (`Writer::deliver`).
    pub(crate) text_ahead: HashMap<NodeId, u32>,
    /// The watched nodes and the visibility facts last reported of each.
    pub(crate) watched: BTreeMap<NodeId, NodeFlags>,
    /// How many times the font database changed.
    pub(crate) font_generation: u64,
}

impl AppModel {
    /// The App of the first window `main`.
    pub(crate) fn new(shared: Shared, main: Model) -> Self {
        let window = main.window;
        let mut windows = BTreeMap::new();
        windows.insert(
            window,
            Window {
                model: main,
                host: super::window::HostWindow::new(kivact_types::WindowMode::Normal, None),
            },
        );
        Self {
            windows,
            events: shared.events,
            writes: Writes::default(),
            assets: shared.assets,
            window_changes: Vec::new(),
            effects: Vec::new(),
            clock: kivact_types::clock::Clock::new(),
            client_frames: FrameRequests::default(),
            focused_window: None,
            external_windows: BTreeMap::new(),
            devtools_changes: Vec::new(),
            log: shared.log,
            injecting: BTreeMap::new(),
            text_ahead: HashMap::new(),
            watched: BTreeMap::new(),
            font_generation: 0,
        }
    }

    /// The model of `id`.
    pub(crate) fn model(&self, id: WindowId) -> Option<&Model> {
        self.windows.get(&id).map(|w| &w.model)
    }

    /// The model of `id`, mutably.
    pub(crate) fn model_mut(&mut self, id: WindowId) -> Option<&mut Model> {
        self.windows.get_mut(&id).map(|w| &mut w.model)
    }

    /// The host state of window `id`.
    pub(crate) fn host(&self, id: WindowId) -> Option<&super::window::HostWindow> {
        self.windows.get(&id).map(|w| &w.host)
    }

    /// The host state of window `id`, mutably.
    pub(crate) fn host_mut(&mut self, id: WindowId) -> Option<&mut super::window::HostWindow> {
        self.windows.get_mut(&id).map(|w| &mut w.host)
    }

    /// Window `id` handles an event or a turn at host time `host_ns`.
    pub(crate) fn set_now(&mut self, id: WindowId, host_ns: u64) {
        let now = super::frame::state::Now::at(host_ns, &self.clock);
        if let Some(window) = self.windows.get_mut(&id) {
            window.model.frame.set_now(now);
        }
    }

    /// The window whose model holds `node`.
    pub(crate) fn window_of(&self, node: NodeId) -> Option<WindowId> {
        self.windows
            .iter()
            .find(|(_, window)| window.model.tree.get(node).is_some())
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
        let model = &mut windows.get_mut(&window)?.model;
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
        let host = super::window::HostWindow::new(request.mode, request.parent);
        self.windows.insert(request.id, Window { model, host });
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
        self.injecting.remove(&id);
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

    /// The window the OS focuses.
    pub(crate) fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Queue `event` after every event queued so far.
    pub(crate) fn push_event(&self, event: HostEvent) {
        self.events.push(event);
    }

    /// The client is handed `events`: their text edits hold back the
    /// input-value echoes of their nodes until each is dispatched.
    pub(crate) fn hold_text(&mut self, events: &[HostEvent]) {
        for event in events {
            if let HostEvent::TextChanged { node, .. } = event {
                *self.text_ahead.entry(*node).or_default() += 1;
            }
        }
    }

    /// The client dispatches `event` now.
    pub(crate) fn delivered(&mut self, event: &HostEvent) {
        if let HostEvent::TextChanged { node, .. } = event
            && let Some(count) = self.text_ahead.get_mut(node)
        {
            *count -= 1;
            if *count == 0 {
                self.text_ahead.remove(node);
            }
        }
    }

    /// Report the watched nodes whose visibility facts changed.
    pub(crate) fn report_flags(&mut self) {
        let ids: Vec<NodeId> = self.watched.keys().copied().collect();
        for id in ids {
            let Some(window) = self.window_of(id) else {
                self.watched.remove(&id);
                continue;
            };
            let flags = self
                .model(window)
                .map(|m| m.node_flags(id))
                .unwrap_or_default();
            if self.watched.get(&id) != Some(&flags) {
                self.watched.insert(id, flags);
                self.push_event(HostEvent::NodeFlagsChanged { node: id, flags });
            }
        }
    }
}
