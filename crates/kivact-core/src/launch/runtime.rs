//! Role: Concrete host callbacks and the ordered App turn loop.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::{Boundary, Client, HostWaker, Writer};
use kivact_engine::contract::{HostEvent, HostInfo, TurnOutput};

use crate::client::state::HostState;

use crate::view::View;
use crate::view::requests;
use crate::view::state::ViewState;
use crate::view::text_edits::TextEditsAhead;
use kivact_runtime::TaskState;
use kivact_runtime::{
    ErrorKind, ExitState, Extensions, Runtime, RuntimeConfig, Stage, ThreadWaker, TurnGuard,
    TurnReport, UiError,
};

/// Rounds of `on_mount` callbacks and user effects one turn runs before
/// the rest is cut as a `Cycle`.
const USER_EFFECT_ROUNDS: usize = 100;

use super::settings::AppOptions;
/// Whether the host started the App.
pub(super) enum LaunchState {
    /// Waiting for the host's `start`, which consumes the config.
    Waiting {
        options: Box<AppOptions>,
        root: Box<dyn FnOnce() -> View>,
    },
    /// The host started the App.
    Started(Started),
}

/// The App side of the host boundary: owns the App's runtime and implements
/// concrete callbacks for the engine window loop or test driver.
pub(super) struct Started {
    rt: Rc<Runtime>,
}

struct ClientHost(HostWaker);
impl kivact_runtime::Host for ClientHost {
    fn wake(&self) {
        self.0.wake();
    }
}

pub struct Launch {
    /// Where the App was started: the location of the root owner.
    at: &'static Location<'static>,
    pub(super) launch: LaunchState,
    /// Host mode and its close/timing protocol for this application.
    pub(super) mode: ClientMode,
}

impl Launch {
    /// Prepare an application; the engine constructs its runtime at start.
    #[track_caller]
    pub fn new(
        options: AppOptions,
        mode: ClientMode,
        root: impl FnOnce() -> View + 'static,
    ) -> Self {
        Self {
            at: Location::caller(),
            launch: LaunchState::Waiting {
                options: Box::new(options),
                root: Box::new(root),
            },
            mode,
        }
    }

    /// The App's runtime (the Testing Library reads the tree and the
    /// extensions through it).
    pub(super) fn runtime(&self) -> &Rc<Runtime> {
        match &self.launch {
            LaunchState::Started(started) => &started.rt,
            LaunchState::Waiting { .. } => {
                panic!("invariant: runtime is available after host start")
            }
        }
    }
    pub(super) fn shared_client(
        shared: Rc<RefCell<Self>>,
        timing: Option<Rc<dyn Fn(u64)>>,
    ) -> Client {
        let start = shared.clone();
        Client::new(
            move |info, waker, writer| start.borrow_mut().start(info, waker, writer),
            move |events, boundaries, now_ns| {
                let started = timing.as_ref().map(|_| std::time::Instant::now());
                let output = shared.borrow_mut().turn(events, boundaries, now_ns);
                if let Some((timing, started)) = timing.as_ref().zip(started) {
                    timing(started.elapsed().as_nanos() as u64);
                }
                output
            },
        )
    }
}

impl Launch {
    fn start(&mut self, info: &HostInfo, waker: HostWaker, writer: Writer) {
        let (options, _) = match &mut self.launch {
            LaunchState::Waiting { options, root } => (options, root),
            LaunchState::Started(_) => unreachable!("invariant: host starts an App once"),
        };
        let ready = waker.clone();
        let rt = Runtime::new(
            Rc::new(ClientHost(waker)),
            ThreadWaker::new(move || ready.wake()),
            RuntimeConfig {
                start_ns: info.now_ns,
                blocking_threads: options.settings.blocking_threads,
                at: self.at,
            },
            |_rt, root| {
                let env = Rc::new(crate::view::state::EnvState::new(
                    root,
                    info.env.clone(),
                    info.surface_active,
                    self.at,
                ));
                let motion_env = kivact_runtime::MotionEnv {
                    reduced_motion: env.field("is_motion_reduced", |e| e.reduced_motion),
                    surface_active: kivact_runtime::Value::from(env.surface_active),
                    frame_period_ns: env.field("frame_period_ns", |e| e.frame_period_ns),
                };
                let view = ViewState::new(
                    root,
                    info,
                    writer.clone(),
                    env,
                    options.settings.min_window_size,
                    self.at,
                );
                let extensions = Extensions::new().with(view).with(motion_env);
                let extensions = extensions.with(crate::gpu_interop::State::new(root, self.at));
                #[cfg(debug_assertions)]
                let extensions = {
                    let trace = crate::client::trace::Trace::default();
                    extensions.with(trace)
                };
                extensions
                    .with(crate::platform::lifecycle::LifecycleState::default())
                    .with(HostState::default())
            },
        );
        let setup = rt.begin_setup();
        let root_owner = rt.root();
        #[cfg(debug_assertions)]
        crate::client::trace::event(&rt, format_args!("host_events start {info:?}"));
        writer.set_window(
            std::mem::take(&mut options.settings.title),
            options.settings.min_window_size,
        );
        // The App's fonts precede the root's mount (the engine registers
        // a font when it is written, so the first layout uses them).
        requests::register_app_fonts(&rt, std::mem::take(&mut options.settings.fonts));
        let families = ViewState::tree(&rt).reader().font_families();
        rt.enter(|| ViewState::of(&rt).fonts_value.set(families));
        let (options, root) = match std::mem::replace(
            &mut self.launch,
            LaunchState::Started(Started { rt: rt.clone() }),
        ) {
            LaunchState::Waiting { options, root } => (*options, root),
            LaunchState::Started(_) => unreachable!("invariant: host starts an App once"),
        };
        crate::view::root::mount_root(&rt, root_owner, options.into_view_settings(), root, self.at);
        // What the platform cannot do is diagnosed, not silently ignored:
        // `tracing` says it at start, and the turn trace when the App
        // relies on it (`platform::safe_area`).
        let missing = crate::platform::missing_capabilities(&ViewState::of(&rt).capabilities());
        if !missing.is_empty() {
            crate::client::notice::inform(&format!("the platform lacks: {}", missing.join(", ")));
        }
        setup.stabilize();
        let written = commit(&setup, &rt, &writer);
        debug_assert!(written > 0, "the first revision sets the window");
        setup.finish_setup();
        #[cfg(debug_assertions)]
        crate::client::trace::record(&rt);
    }

    /// One turn: every queued event, then everything
    /// they cause (tasks and timers, activations, commits, `on_mount` and
    /// user effects) until nothing is left, the writes committed. The
    /// boundaries among the events only decide when the `on_mount`
    /// callbacks of their revisions run.
    fn turn(&mut self, events: &[HostEvent], boundaries: &[Boundary], now_ns: u64) -> TurnOutput {
        let rt = self.runtime().clone();
        let host = HostState::of(&rt);
        rt.advance_clock(now_ns);
        let tree = ViewState::tree(&rt).engine().clone();
        // Commit boundaries that carry no event and release no callback
        // make no turn.
        if events.is_empty()
            && boundaries
                .last()
                .is_some_and(|b| !rt.mounts_due(b.revision))
        {
            return output(rt.report_now());
        }
        // The revisions done before the events: all but those whose
        // boundaries this turn delivers.
        let before = boundaries
            .first()
            .map_or(tree.stats().revision, |b| b.revision.0.saturating_sub(1));
        let mut stats = host.stats.get();
        stats.turns += 1;
        host.stats.set(stats);
        let turn = rt.begin_turn(events.len());
        ViewState::of(&rt).initial_render.set(false);
        turn.run_mounts(kivact_types::Revision(before));
        turn.deliver_reports();
        turn.stabilize();
        let edits = TextEditsAhead::begin(&mut ViewState::tree_mut(&rt), events);
        let mut boundaries = boundaries.iter().peekable();
        let mut acknowledge = |delivered: usize| {
            while let Some(b) = boundaries.next_if(|b| b.events <= delivered) {
                turn.run_mounts(b.revision);
                turn.stabilize();
            }
        };
        acknowledge(0);
        let mut delivered = 0;
        for event in events {
            if matches!(&self.mode, ClientMode::Embedded(closing) if closing.is_set()) {
                break;
            }
            turn.stage(Stage::Dispatch);
            #[cfg(debug_assertions)]
            crate::client::trace::event(&rt, format_args!("host_events receive {event:?}"));
            edits.deliver(event);
            turn.cause(describe(event));
            crate::client::dispatch::dispatch(&rt, event);
            if crate::client::dispatch::run_requested_activations(&rt) {
                turn.stabilize();
            }
            turn.stabilize();
            delivered += 1;
            acknowledge(delivered);
        }
        acknowledge(events.len());
        let done = kivact_types::Revision(tree.stats().revision);
        turn.check_draining();
        turn.settle_async();
        let writes = settle_user_effects(&turn, &rt, &tree, done);

        let output = output(turn.finish(writes, requests::has_activations(&rt)));
        #[cfg(debug_assertions)]
        {
            crate::client::trace::record(&rt);
            crate::client::trace::event(&rt, format_args!("host_events output {output:?}"));
        }
        output
    }
}
fn settle_user_effects(
    turn: &TurnGuard<'_>,
    rt: &Runtime,
    tree: &Writer,
    done: kivact_types::Revision,
) -> usize {
    let mut writes = 0;
    let mut round = 0;
    loop {
        if crate::client::dispatch::run_requested_activations(rt) {
            turn.stabilize();
            turn.settle_async();
        }
        writes += commit(turn, rt, tree);
        let mounts = rt.mounts_due(done);
        // The commit may queue render work too (a failed focus request
        // reaching an error boundary): it runs and commits in this turn.
        let pending = rt.pending();
        if !mounts && !pending.user && !pending.render {
            break;
        }
        round += 1;
        if round > USER_EFFECT_ROUNDS {
            let abandoned = turn.abandon_user_effects();
            let path: Vec<_> = abandoned
                .iter()
                .filter_map(|k| rt.node_info(*k).map(|n| n.at))
                .collect();
            rt.report(
                None,
                UiError::new(
                    ErrorKind::Cycle { path: path.clone() },
                    "user effects did not settle within 100 rounds",
                    path.first().copied(),
                ),
            );
            break;
        }
        if mounts {
            turn.run_mounts(done);
            rt.stabilize();
        }
        turn.run_user_effects();
        rt.stabilize();
        turn.settle_async();
    }
    writes
}

fn output(report: TurnReport) -> TurnOutput {
    TurnOutput {
        next_wake_ns: report.next_deadline,
        exit: matches!(report.exit, ExitState::Exited),
    }
}

impl Drop for Launch {
    fn drop(&mut self) {
        let LaunchState::Started(started) = &self.launch else {
            return;
        };
        let rt = &started.rt;
        let tasks = TaskState::of(rt);
        // A propagating panic ends the App: run no more App code. Only the
        // worker threads are stopped.
        if !std::thread::panicking() {
            rt.root().dispose();
            tasks.abandon_all_queues();
            // async-task schedules cancellation when its handle is dropped.
            // Release those futures on the UI thread before the executor
            // stops; their channel wakers can otherwise retain its queue and
            // receivers.
            if tasks.has_ready() {
                tasks.poll_tasks(rt);
            }
        }
        tasks.shutdown();
        if !std::thread::panicking() {
            rt.shutdown();
        }
        // Opening may fail before an Embedded is returned, or its drop may
        // precede deferred callback cleanup. Record task retention here too.
        if matches!(self.mode, ClientMode::Embedded(_))
            && kivact_types::process::DetachedThreads::any()
        {
            kivact_engine::embed::retain_module();
        }
    }
}

/// The name of a pointer phase without its press data (trace labels carry
/// no values).
fn phase_name(phase: kivact_types::PointerPhase) -> &'static str {
    use kivact_types::PointerPhase as P;
    match phase {
        P::Down(_) => "Down",
        P::Move => "Move",
        P::Up(_) => "Up",
        P::Cancel => "Cancel",
        P::Enter => "Enter",
        P::Leave => "Leave",
    }
}

/// Short cause label of an event for the structured trace (no values).
fn describe(e: &kivact_engine::contract::HostEvent) -> String {
    use kivact_engine::contract::HostEvent as E;
    match e {
        E::Pointer(route) => format!(
            "Pointer {} {:?}",
            phase_name(route.input.phase),
            route.target
        ),
        E::Wheel(route) => format!("Wheel {:?}", route.target),
        E::Key(route) => format!(
            "Key {:?} {:?} region {:?}",
            route.input.key, route.target, route.region
        ),
        E::Activate { node, region, .. } => {
            format!("Activate n{} region {:?}", node.0, region.map(|r| r.0))
        }
        E::FocusChanged {
            node,
            region,
            visible,
        } => format!(
            "FocusChanged {:?} region {:?} visible {visible}",
            node.map(|n| n.0),
            region.map(|r| r.0)
        ),
        E::HoverChanged { target } => {
            format!("HoverChanged {:?}", target.map(|t| (t.node.0, t.region)))
        }
        E::MousePosition(position) => format!("MousePosition {position:?}"),
        E::TextChanged { node, .. } => format!("TextChanged n{}", node.0),
        E::SelectionChanged { node, .. } => format!("SelectionChanged n{}", node.0),
        E::Ime { node, .. } => format!("Ime n{}", node.0),
        E::ScrollChanged { node, .. } => format!("ScrollChanged n{}", node.0),
        E::LayoutChanged { node, .. } => format!("LayoutChanged n{}", node.0),
        E::A11y { node, request, .. } => format!("A11y {:?} n{}", request.action(), node.0),
        E::SnapshotResult { request, .. } => format!("SnapshotResult {}", request.0),
        other => {
            let s = format!("{other:?}");
            s.split([' ', '{', '('])
                .next()
                .unwrap_or_default()
                .to_string()
        }
    }
}

fn commit(turn: &TurnGuard<'_>, rt: &Runtime, writer: &Writer) -> usize {
    turn.stage(Stage::Commit);
    if let Some(at_ns) = turn.plan_frame() {
        writer.request_frame(at_ns);
    }
    requests::flush_focus(rt);
    let host = HostState::of(rt);
    let removed = std::mem::take(&mut *ViewState::of(rt).removed.borrow_mut());
    if !removed.is_empty() {
        host.asset_failures
            .borrow_mut()
            .retain(|(id, _)| !removed.contains(id));
    }
    let writes = writer.pending_writes();
    // The new `on_mount` callbacks wait for the revision these writes
    // become, or for the last one when nothing was written.
    let revision = if writes == 0 {
        writer.stats().revision
    } else {
        writer.commit().0
    };
    #[cfg(debug_assertions)]
    {
        crate::client::trace::stage(rt, "Commit");
        crate::client::trace::event(
            rt,
            format_args!("host_events commit revision={revision} writes={writes}"),
        );
    }
    turn.stamp_mounts(kivact_types::Revision(revision));
    writes
}

/// The engine host driving this application.
pub enum ClientMode {
    Window,
    Embedded(kivact_engine::contract::CloseSignal),
    Test(TestTiming),
}
/// Reports the duration of each core turn to the test harness.
pub struct TestTiming(pub Rc<dyn Fn(u64)>);

pub(crate) fn run_native(
    client: Client,
    options: kivact_engine::api::HostOptions,
) -> crate::Result<()> {
    crate::platform::run_native(client, options)
}
