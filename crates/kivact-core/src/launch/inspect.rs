//! Role: Read-only inspection of a started application.
use super::runtime::{ClientMode, Launch, LaunchState, TestTiming};
use crate::{client::state::HostState, view::state::ViewState};
use kivact_engine::api::Client;
use kivact_runtime::{Runtime, TaskState, UiError};
use std::{cell::RefCell, panic::Location, rc::Rc};
/// Read-only handle shared with an engine client.
pub struct LaunchHandle(pub(super) Rc<RefCell<Launch>>);
pub struct Inspect<'a> {
    launch: std::cell::Ref<'a, Launch>,
}
impl LaunchHandle {
    pub fn inspect(&self) -> Option<Inspect<'_>> {
        let launch = self.0.borrow();
        if matches!(launch.launch, LaunchState::Started(_)) {
            Some(Inspect { launch })
        } else {
            None
        }
    }
}
impl Inspect<'_> {
    fn rt(&self) -> &Runtime {
        self.launch.runtime()
    }
    pub fn reader(&self) -> kivact_engine::api::Reader {
        ViewState::tree(self.rt()).reader()
    }
    pub fn root(&self) -> kivact_types::NodeId {
        ViewState::tree(self.rt()).root
    }
    pub fn app_view(&self) -> Option<kivact_types::NodeId> {
        self.reader().children(self.root()).first().copied()
    }
    pub fn located_at(&self, node: kivact_types::NodeId) -> Option<&'static Location<'static>> {
        ViewState::tree(self.rt()).get(node).and_then(|n| n.at)
    }
    pub fn contains(&self, node: kivact_types::NodeId) -> bool {
        ViewState::tree(self.rt()).contains(node)
    }
    pub fn why(&self, node: kivact_types::NodeId) -> crate::diagnostics::Why {
        crate::diagnostics::why::explain(self.rt(), node)
    }
    pub fn turns(&self) -> Vec<kivact_runtime::TurnRecord> {
        self.rt().turns()
    }
    #[cfg(debug_assertions)]
    pub fn open_trace(&self, name: &str) {
        crate::client::trace::Trace::of(self.rt()).open(name);
    }
}

impl Inspect<'_> {
    pub fn stats(&self) -> crate::diagnostics::Stats {
        let ts = ViewState::tree(self.rt()).engine().stats();
        let host = HostState::of(self.rt());
        let live = self.rt().live_counts();
        let motion = kivact_runtime::MotionState::of(self.rt()).stats();
        crate::diagnostics::Stats {
            live_reactive: live.reactive,
            live_owners: live.owners,
            live_objects: live.objects,
            live_constants: live.constants,
            arena_slots: live.slots,
            tree_nodes: ViewState::tree(self.rt()).len(),
            timers: TaskState::of(self.rt()).stats().timers,
            frame_loops: motion.loops,
            animations: motion.animations,
            batches: ts.batches,
            a11y_constructed: ts.a11y_constructed,
            a11y_published: ts.a11y_published,
            ops: ts.ops,
            last_batch_ops: ts.last_ops,
            creates: ts.creates,
            set_props: ts.set_props,
            structural: ts.structural,
            host_touched_last: ts.touched_last,
            turns: host.turns(),
            stale_events: host.stale_events(),
            core_ns: 0,
        }
    }
}

impl Inspect<'_> {
    pub fn errors(&self) -> Vec<UiError> {
        self.rt().errors()
    }
    pub fn pending(&self) -> bool {
        let pending = self.rt().pending();
        pending.turn_requested
            || pending.user
            || pending.render
            || TaskState::of(self.rt()).has_ready()
    }
    pub fn outstanding_blocking(&self) -> usize {
        TaskState::of(self.rt()).stats().outstanding_blocking
    }
    pub fn motion_ticks(&self) -> u64 {
        kivact_runtime::MotionState::of(self.rt()).stats().ticks
    }
}

impl Launch {
    pub fn into_engine_client(self) -> (Client, LaunchHandle) {
        let timing = match &self.mode {
            ClientMode::Test(TestTiming(timing)) => Some(timing.clone()),
            _ => None,
        };
        let shared = Rc::new(RefCell::new(self));
        (
            Self::shared_client(shared.clone(), timing),
            LaunchHandle(shared),
        )
    }
}
