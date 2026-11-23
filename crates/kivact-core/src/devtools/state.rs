//! Role: The DevTools state of the App's runtime, shared by hooks and leaves.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::panic::Location;
use std::rc::Rc;

use kivact_devtools_agent::{Agent, Responder};
use kivact_devtools_protocol as p;
use kivact_engine::api::Prop;
use kivact_engine::contract::{DevtoolsOverlay, PickPhase};
use kivact_runtime::NodeId as EffectId;
use kivact_runtime::driver::{LiveSnapshot, Runtime};
use kivact_runtime::{Owner, Signal};
use kivact_types::cell::{LocalMailbox, ReportOnce};
use kivact_types::{FocusTarget, NodeId, RequestId, Revision, StateSet, WindowId};

use super::inspect_mode;
use crate::view::devtools_hooks::NodeOverrides;

/// The DevTools state of an App: off, or the running agent and its data.
pub(crate) enum DevtoolsState {
    Off,
    Running(Rc<Inspector>),
}

/// The agent of an App whose launch allows DevTools.
pub(crate) enum AgentSlot {
    /// Starting on a background thread; the first turn after it is ready promotes it.
    Starting(std::sync::mpsc::Receiver<std::io::Result<Agent>>),
    Ready(Box<Agent>),
    /// It could not start (already warned).
    Failed,
}

/// What a launch that allows DevTools holds until the agent's background
/// start finishes: the receiver of the started agent.
pub(crate) type AgentStart = std::sync::mpsc::Receiver<std::io::Result<Agent>>;

pub(crate) struct Inspector {
    pub(crate) agent: RefCell<AgentSlot>,
    /// What the view hooks read and write.
    pub(crate) hooks: HookData,
    /// What only devtools/* uses.
    pub(crate) session: SessionData,
}

// xtask-allow(state): record: DevTools hook tables with separate writers: recording (updates start and stop), prop_writes (the prop_written hook, drained by updates), overrides and forced (styles calls; forced is pruned by the turn), emulation (emulation calls), focus_history (the focus_changed hook, swept per closed window), input_log with its sequence allocator next_input (input_event); node_overrides is the projection of overrides written only by rebuild_node_overrides
pub(crate) struct HookData {
    pub(crate) recording: RefCell<p::RecordingSettings>,
    pub(crate) prop_writes: RefCell<Vec<PropWrite>>,
    pub(crate) overrides: Signal<Vec<p::StyleOverride>>,
    pub(crate) forced: Signal<BTreeMap<NodeId, StateSet>>,
    pub(crate) emulation: Signal<p::Emulation>,
    pub(crate) focus_history: RefCell<BTreeMap<WindowId, VecDeque<p::FocusMove>>>,
    pub(crate) programmatic_focus: LocalMailbox<FocusTarget>,
    pub(crate) input_log: RefCell<VecDeque<p::InputLogEntry>>,
    /// The last `InputLogEntry.seq` given (S-8 id allocator; 1-based, in log order).
    pub(crate) next_input: Cell<u64>,
    /// The style overrides parsed once when they change, read by the style hooks.
    pub(crate) node_overrides: Rc<RefCell<NodeOverrides>>,
    pub(crate) node_overrides_changed: kivact_runtime::Trigger,
}

// xtask-allow(state): record: DevTools session tables with separate writers: events (the host_event hook, taken by the turn), pending (find, inject and list_turns calls), pending_graphs and turns (the graph pipeline of updates), frames and update_counts (records), inspector (pick mode), overlays and sent_overlays (settings and what was last sent to the engine), snapshots (perf), sent_windows (tree) and errors_sent (console); next_override and next_snapshot only allocate ids
pub(crate) struct SessionData {
    pub(crate) events: RefCell<Vec<DevtoolsEvent>>,
    pub(crate) pending: RefCell<Vec<Pending>>,
    pub(crate) pending_graphs: RefCell<VecDeque<PendingGraph>>,
    pub(crate) turns: RefCell<VecDeque<p::TurnGraph>>,
    pub(crate) frames: RefCell<BTreeMap<WindowId, VecDeque<p::FrameInfo>>>,
    pub(crate) update_counts: RefCell<HashMap<NodeId, u32>>,
    pub(crate) next_override: Cell<u64>,
    pub(crate) inspector: Cell<inspect_mode::Inspector>,
    pub(crate) overlays: RefCell<p::OverlaySettings>,
    pub(crate) sent_overlays: RefCell<BTreeMap<WindowId, DevtoolsOverlay>>,
    pub(crate) snapshots: RefCell<VecDeque<(u64, LiveSnapshot)>>,
    pub(crate) next_snapshot: Cell<u64>,
    /// The open windows the last turn saw, and the list last sent.
    pub(crate) sent_windows: RefCell<SentWindows>,
    /// The last error sequence number sent to the Errors subscribers.
    pub(crate) errors_sent: Cell<u64>,
    pub(crate) open_failed: ReportOnce,
}

/// What the last turn saw of the windows: the open windows, for sweeping the
/// records of a closed one, and the list last sent to the Windows subscribers,
/// so a title, size or focus change is sent too.
#[derive(Default)]
pub(crate) struct SentWindows {
    pub(crate) open: Vec<WindowId>,
    pub(crate) listed: Option<Vec<p::WindowInfo>>,
}

impl SessionData {
    /// Leave pick mode; the node the pointer last hovered stays the
    /// highlight, as it did before pick mode began.
    pub(crate) fn end_pick(&self) {
        let highlight = self.inspector.get().highlight();
        self.inspector
            .set(inspect_mode::Inspector::Idle { highlight });
    }
}

pub(crate) struct PropWrite {
    pub(crate) node: NodeId,
    pub(crate) prop: Prop,
    pub(crate) effect: Option<EffectId>,
    pub(crate) changed: bool,
}

/// A call answered after a later turn: `find_*`, `list_turns` with `wait_ms`, and `inject_input` wait.
/// Each entry carries its window, so closing the window answers it.
pub(crate) enum Pending {
    Find {
        by: QueryBy,
        window: WindowId,
        deadline_ns: u64,
        reply: Responder,
    },
    Inject {
        window: WindowId,
        request: RequestId,
        reply: Responder,
    },
    Turns {
        params: p::ListTurnsParams,
        deadline_ns: u64,
        reply: Responder,
    },
}

impl Pending {
    /// The window the answer waits for; `None` for a call that is not tied to one.
    pub(crate) fn window(&self) -> Option<WindowId> {
        match self {
            Pending::Find { window, .. } | Pending::Inject { window, .. } => Some(*window),
            Pending::Turns { .. } => None,
        }
    }
}

pub(crate) enum QueryBy {
    Role(p::RoleQuery),
    Text(p::TextQuery),
    TestId(p::TestIdQuery),
}

/// A turn graph waiting for the layout and frame of its revision (completed by
/// its frame, by any record of a newer revision, or by `stop_recording`).
pub(crate) struct PendingGraph {
    pub(crate) revision: Revision,
    pub(crate) graph: p::TurnGraph,
}

/// An engine event for the DevTools, queued by the hook the client dispatch calls
/// and handled by `devtools::run_events` (the client does not reference `devtools`).
pub(crate) enum DevtoolsEvent {
    Key {
        window: WindowId,
        pick: bool,
    },
    Pick {
        window: WindowId,
        node: Option<NodeId>,
        phase: PickPhase,
    },
    InjectDelivered {
        request: RequestId,
    },
}

impl DevtoolsState {
    /// The running inspector; a call arrives only while the agent runs.
    #[inline]
    pub(crate) fn inspector(rt: &Runtime) -> &Inspector {
        match Self::of(rt) {
            DevtoolsState::Running(inspector) => inspector,
            DevtoolsState::Off => {
                unreachable!("invariant: a call arrives only while the agent runs")
            }
        }
    }

    /// Build the state of an App from the agent slot the launch asked for.
    pub(crate) fn from_slot(
        root: Owner,
        slot: Option<AgentStart>,
        at: &'static Location<'static>,
    ) -> DevtoolsState {
        let Some(receiver) = slot else {
            return DevtoolsState::Off;
        };
        let overrides: Signal<Vec<p::StyleOverride>> = root.signal(Vec::new(), at);
        let forced: Signal<BTreeMap<NodeId, StateSet>> = root.signal(BTreeMap::new(), at);
        let emulation: Signal<p::Emulation> = root.signal(emulation_off(), at);
        DevtoolsState::Running(Rc::new(Inspector {
            agent: RefCell::new(AgentSlot::Starting(receiver)),
            hooks: HookData {
                recording: RefCell::new(recording_off()),
                prop_writes: RefCell::new(Vec::new()),
                overrides,
                forced,
                emulation,
                focus_history: RefCell::new(BTreeMap::new()),
                programmatic_focus: LocalMailbox::new(),
                input_log: RefCell::new(VecDeque::new()),
                next_input: Cell::new(0),
                node_overrides: Rc::default(),
                node_overrides_changed: root.trigger(at),
            },
            session: SessionData {
                events: RefCell::new(Vec::new()),
                pending: RefCell::new(Vec::new()),
                pending_graphs: RefCell::new(VecDeque::new()),
                turns: RefCell::new(VecDeque::new()),
                frames: RefCell::new(BTreeMap::new()),
                update_counts: RefCell::new(HashMap::new()),
                next_override: Cell::new(0),
                inspector: Cell::new(inspect_mode::Inspector::Idle { highlight: None }),
                overlays: RefCell::new(overlays_off()),
                sent_overlays: RefCell::new(BTreeMap::new()),
                snapshots: RefCell::new(VecDeque::new()),
                next_snapshot: Cell::new(0),
                sent_windows: RefCell::default(),
                errors_sent: Cell::new(0),
                open_failed: ReportOnce::new(),
            },
        }))
    }
}

impl DevtoolsState {
    /// The hooks the view calls; `None` while DevTools is off.
    pub(crate) fn hooks(&self) -> Option<Rc<dyn crate::view::devtools_hook_traits::DevtoolsHooks>> {
        match self {
            DevtoolsState::Off => None,
            DevtoolsState::Running(i) => {
                Some(i.clone() as Rc<dyn crate::view::devtools_hook_traits::DevtoolsHooks>)
            }
        }
    }
}

impl Inspector {
    /// The ready agent; `None` while it starts or after it failed.
    pub(crate) fn agent(&self) -> Option<std::cell::RefMut<'_, Agent>> {
        std::cell::RefMut::filter_map(self.agent.borrow_mut(), |slot| match slot {
            AgentSlot::Ready(agent) => Some(&mut **agent),
            AgentSlot::Starting(_) | AgentSlot::Failed => None,
        })
        .ok()
    }

    /// The ready agent, for a call that only a ready agent delivers.
    pub(crate) fn ready_agent(&self) -> std::cell::RefMut<'_, Agent> {
        self.agent()
            .expect("invariant: calls arrive only from a ready agent")
    }

    /// Whether a connection subscribes to `topic`.
    pub(crate) fn wants(&self, topic: p::Topic) -> bool {
        self.agent().is_some_and(|agent| agent.wants(topic))
    }

    /// Send a notification to the connections subscribed to its topic.
    pub(crate) fn notify(&self, notification: p::Notification) {
        if let Some(agent) = self.agent() {
            agent.notify(notification);
        }
    }
}

/// The emulation of an App without DevTools.
pub(crate) fn emulation_off() -> p::Emulation {
    p::Emulation {
        theme: None,
        reduced_motion: None,
        locale: None,
        pseudo_locale: p::PseudoLocale::Off,
        scale: None,
        touch: None,
    }
}

/// The recording settings of an App that is not recording.
pub(crate) fn recording_off() -> p::RecordingSettings {
    p::RecordingSettings {
        updates: false,
        frames: false,
        input: false,
    }
}

/// The overlays of an App without DevTools.
pub(crate) fn overlays_off() -> p::OverlaySettings {
    p::OverlaySettings {
        paint_flashing: false,
        layout_lint: false,
        update_heatmap: false,
        tab_order: false,
        selected_only: false,
    }
}
