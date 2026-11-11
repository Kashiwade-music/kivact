//! Role: The DevTools state of the App's runtime, shared by hooks and leaves.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::panic::Location;

use kivact_devtools_agent::{Agent, Responder};
use kivact_devtools_protocol as p;
use kivact_engine::api::Prop;
use kivact_engine::contract::{DevtoolsOverlay, PickPhase};
use kivact_runtime::NodeId as EffectId;
use kivact_runtime::{LiveSnapshot, Owner, Runtime, Signal};
use kivact_style::internal::ForcedStates;
use kivact_types::cell::{LocalMailbox, ReportOnce, Sent};
use kivact_types::{FocusTarget, NodeId, RequestId, Revision, WindowId};

use super::inspect_mode;

/// The DevTools state of an App: off, or the running agent and its data.
pub(crate) enum DevtoolsState {
    Off,
    Running(Box<Inspector>),
}

/// What `devtools::start::agent_slot` returns.
#[allow(
    clippy::large_enum_variant,
    reason = "one agent per process, moved once at startup; boxing would only add noise"
)]
pub(crate) enum AgentSlot {
    Off,
    Running { agent: Agent, can_inject: bool },
}

pub(crate) struct Inspector {
    pub(crate) agent: RefCell<Agent>,
    pub(crate) can_inject: bool,
    /// What the view hooks read and write.
    pub(crate) hooks: HookData,
    /// What only devtools/* uses.
    pub(crate) session: SessionData,
}

pub(crate) struct HookData {
    pub(crate) recording: RefCell<p::RecordingSettings>,
    pub(crate) prop_writes: RefCell<Vec<PropWrite>>,
    pub(crate) overrides: Signal<Vec<p::StyleOverride>>,
    pub(crate) forced: Signal<BTreeMap<NodeId, ForcedStates>>,
    pub(crate) emulation: Signal<p::Emulation>,
    pub(crate) focus_history: RefCell<BTreeMap<WindowId, VecDeque<p::FocusMove>>>,
    pub(crate) programmatic_focus: LocalMailbox<FocusTarget>,
    pub(crate) input_log: RefCell<VecDeque<p::InputLogEntry>>,
    /// The last `InputLogEntry.seq` given (S-8 id allocator; 1-based, in log order).
    pub(crate) next_input: Cell<u64>,
}

pub(crate) struct SessionData {
    pub(crate) events: RefCell<Vec<DevtoolsEvent>>,
    pub(crate) pending: RefCell<Vec<Pending>>,
    pub(crate) pending_graphs: RefCell<VecDeque<PendingGraph>>,
    pub(crate) turns: RefCell<VecDeque<p::TurnGraph>>,
    pub(crate) frames: RefCell<BTreeMap<WindowId, VecDeque<p::FrameInfo>>>,
    pub(crate) update_counts: RefCell<HashMap<NodeId, u32>>,
    // xtask-allow(state-fields): identity: allocates style override ids
    pub(crate) next_override: Cell<u64>,
    pub(crate) inspector: Cell<inspect_mode::Inspector>,
    pub(crate) overlays: RefCell<p::OverlaySettings>,
    pub(crate) sent_overlays: RefCell<BTreeMap<WindowId, DevtoolsOverlay>>,
    pub(crate) snapshots: RefCell<VecDeque<(u64, LiveSnapshot)>>,
    // xtask-allow(state-fields): identity: allocates snapshot ids
    pub(crate) next_snapshot: Cell<u64>,
    pub(crate) sent_tree: RefCell<HashMap<NodeId, p::ElementInfo>>,
    // xtask-allow(state-fields): cache: the last windows report sent; the whole report is compared with the next one
    pub(crate) sent_windows: RefCell<Sent<Vec<p::WindowInfo>>>,
    // xtask-allow(state-fields): cache: the last error count sent; the number is compared with the next one
    pub(crate) errors_sent: RefCell<Sent<u64>>,
    pub(crate) open_failed: ReportOnce,
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
    // xtask-allow(state-fields): statistic: a diagnostic record of the effect that wrote the prop; known only for effect-driven writes
    pub(crate) effect: Option<EffectId>,
    // xtask-allow(state-fields): independent: whether this one write changed the node's value (the engine's comparison); a write from an effect and a direct write can each change the value or leave it equal, and DevTools shows all four
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
    /// The DevTools state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &DevtoolsState {
        rt.ext::<DevtoolsState>()
    }

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
        slot: AgentSlot,
        at: &'static Location<'static>,
    ) -> DevtoolsState {
        let AgentSlot::Running { agent, can_inject } = slot else {
            return DevtoolsState::Off;
        };
        let overrides: Signal<Vec<p::StyleOverride>> = root.signal(Vec::new(), at);
        let forced: Signal<BTreeMap<NodeId, ForcedStates>> = root.signal(BTreeMap::new(), at);
        let emulation: Signal<p::Emulation> = root.signal(emulation_off(), at);
        DevtoolsState::Running(Box::new(Inspector {
            agent: RefCell::new(agent),
            can_inject,
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
                sent_tree: RefCell::new(HashMap::new()),
                sent_windows: RefCell::new(Sent::new()),
                errors_sent: RefCell::new(Sent::new()),
                open_failed: ReportOnce::new(),
            },
        }))
    }
}

impl Inspector {
    /// Whether a connection subscribes to `topic`.
    pub(crate) fn wants(&self, topic: p::Topic) -> bool {
        self.agent.borrow().wants(topic)
    }

    /// Send a notification to the connections subscribed to its topic.
    pub(crate) fn notify(&self, notification: p::Notification) {
        self.agent.borrow().notify(notification)
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
