//! Role: Concrete host callbacks and the ordered App turn loop.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;
#[cfg(feature = "embed")]
use std::sync::Arc;

use kivact_engine::api::{Boundary, Client, HostWaker, Writer};
use kivact_engine::contract::{HostEvent, HostInfo, TurnOutput};
use kivact_types::Size;

use crate::client::state::HostState;
use crate::i18n::Messages;
use crate::view::View;
use crate::view::requests;
use crate::view::root::AppSettings;
use crate::view::state::ViewState;
use crate::view::text_edits::TextEditsAhead;
use kivact_runtime::TaskState;
use kivact_runtime::{
    ErrorKind, ExitState, Extensions, Runtime, RuntimeConfig, Stage, ThreadWaker, TurnGuard,
    TurnReport, UiError,
};
use kivact_style::Theme;
use kivact_style::asset::Asset;

/// Rounds of `on_mount` callbacks and user effects one turn runs before
/// the rest is cut as a `Cycle`.
const USER_EFFECT_ROUNDS: usize = 100;

/// What an App starts with (its `App` settings and root view);
/// `AppClient::start` consumes it.
pub(crate) struct AppConfig {
    pub(crate) title: String,
    pub(crate) settings: AppSettings,
    /// Fonts registered before the root is mounted (`App::font`): their
    /// `RegisterFont` effects come first in the first revision, so the first
    /// layout measures with them. A rejected font is reported as
    /// `ErrorKind::Asset`.
    pub(crate) fonts: Vec<(Asset, &'static Location<'static>)>,
    /// The window's minimum size (`None`: no minimum), validated by `App::run`.
    pub(crate) min_window_size: Option<Size>,
    /// Builds the root view, in a setup frame of the root owner.
    pub(crate) root: Box<dyn FnOnce() -> View>,
}

impl AppConfig {
    /// An App titled `title` with `theme`, the host's locale, the default
    /// messages, no fonts and no minimum size.
    pub(crate) fn new(title: String, theme: Theme, root: Box<dyn FnOnce() -> View>) -> Self {
        AppConfig {
            title,
            settings: AppSettings {
                theme,
                locale: None,
                messages: Messages::new(),
            },
            fonts: Vec::new(),
            min_window_size: None,
            root,
        }
    }
}

/// Whether the host started the App.
enum Launch {
    /// Waiting for the host's `start`, which consumes the config.
    Waiting {
        config: Box<AppConfig>,
        threads: Option<usize>,
    },
    /// The host started the App.
    Started(Started),
}

/// The App side of the host boundary: owns the App's runtime and implements
/// concrete callbacks for the engine window loop or test driver.
struct Started {
    rt: Rc<Runtime>,
}

struct ClientHost(HostWaker);
impl kivact_runtime::Host for ClientHost {
    fn wake(&self) {
        self.0.wake();
    }
}

pub(crate) struct AppClient {
    #[cfg(debug_assertions)]
    trace_name: Option<String>,
    /// Where the App was started: the location of the root owner.
    at: &'static Location<'static>,
    launch: Launch,
    /// The embedder's close flag, set by `Embedded::drop`: the turn in
    /// progress finishes, but the rest of its event batch is not delivered.
    /// `None`: the App has a window of its own.
    #[cfg(feature = "embed")]
    embed_closing: Option<Arc<std::sync::atomic::AtomicBool>>,
}

impl AppClient {
    /// An App started with `config` when the host starts; `threads` sizes
    /// the blocking pool (default `min(4, CPUs)`). `at` is where the App
    /// was started (`App::run`, `test::render`): diagnostics name the root
    /// owner by it.
    pub(crate) fn new(
        config: AppConfig,
        threads: Option<usize>,
        at: &'static Location<'static>,
    ) -> Self {
        AppClient {
            at,
            launch: Launch::Waiting {
                config: Box::new(config),
                threads,
            },
            #[cfg(debug_assertions)]
            trace_name: None,
            #[cfg(feature = "embed")]
            embed_closing: None,
        }
    }

    #[cfg(feature = "embed")]
    pub(crate) fn with_embed_closing(
        mut self,
        closing: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        self.embed_closing = Some(closing);
        self
    }

    fn embed_closing(&self) -> bool {
        #[cfg(feature = "embed")]
        if let Some(closing) = &self.embed_closing {
            return closing.load(std::sync::atomic::Ordering::Acquire);
        }
        false
    }

    /// The App's runtime (the Testing Library reads the tree and the
    /// extensions through it).
    pub(crate) fn runtime(&self) -> &Rc<Runtime> {
        match &self.launch {
            Launch::Started(started) => &started.rt,
            Launch::Waiting { .. } => panic!("invariant: runtime is available after host start"),
        }
    }
    #[cfg(debug_assertions)]
    pub(crate) fn set_trace_name(&mut self, name: String) {
        self.trace_name = Some(name);
    }

    /// Adapt the runtime owner to the engine's concrete callback bundle.
    pub(crate) fn into_client(self, timing: Option<Rc<dyn Fn(u64)>>) -> Client {
        Self::shared_client(Rc::new(RefCell::new(self)), timing)
    }
    pub(crate) fn shared_client(
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

impl AppClient {
    fn start(&mut self, info: &HostInfo, waker: HostWaker, writer: Writer) {
        let (config, threads) = match &mut self.launch {
            Launch::Waiting { config, threads } => (config, *threads),
            Launch::Started(_) => unreachable!("invariant: host starts an App once"),
        };
        #[cfg(debug_assertions)]
        let trace_name = self.trace_name.clone();
        let ready = waker.clone();
        let rt = Runtime::new(
            Rc::new(ClientHost(waker)),
            ThreadWaker::new(move || ready.wake()),
            RuntimeConfig {
                start_ns: info.now_ns,
                blocking_threads: threads,
                at: self.at,
            },
            |_rt, _root| {
                let extensions = Extensions::new().with(ViewState::new());
                #[cfg(feature = "gpu-interop")]
                let extensions = extensions.with(crate::gpu_interop::State::default());
                #[cfg(debug_assertions)]
                let extensions = {
                    let trace = crate::client::trace::Trace::default();
                    if let Some(name) = &trace_name {
                        trace.open(name);
                    }
                    extensions.with(trace)
                };
                extensions
                    .with(crate::platform::lifecycle::LifecycleState::default())
                    .with(HostState::default())
            },
        );
        let setup = rt.begin_setup();
        let root_owner = rt.root();
        // The tree (with the root node), the environment, the focus source
        // and the capabilities exist before anything writes or reads them.
        crate::view::root::start(
            &rt,
            root_owner,
            info,
            writer.clone(),
            config.min_window_size,
            self.at,
        );
        #[cfg(debug_assertions)]
        crate::client::trace::event(&rt, format_args!("host_events start {info:?}"));
        writer.set_window(std::mem::take(&mut config.title), config.min_window_size);
        // The App's fonts precede the root's mount (the engine registers
        // a font when it is written, so the first layout uses them).
        requests::register_app_fonts(&rt, std::mem::take(&mut config.fonts));
        let config = match std::mem::replace(
            &mut self.launch,
            Launch::Started(Started { rt: rt.clone() }),
        ) {
            Launch::Waiting { config, .. } => *config,
            Launch::Started(_) => unreachable!("invariant: host starts an App once"),
        };
        crate::view::root::mount_root(&rt, root_owner, config.settings, config.root, self.at);
        // What the platform cannot do is diagnosed, not silently ignored:
        // `tracing` says it at start, and the turn trace when the App
        // relies on it (`platform::safe_area`).
        let missing = crate::platform::missing_capabilities(&ViewState::of(&rt).capabilities());
        if !missing.is_empty() {
            super::notice::inform(&format!("the platform lacks: {}", missing.join(", ")));
        }
        setup.stabilize();
        let written = commit(&setup, &rt, &writer);
        debug_assert!(written > 0, "the first revision sets the window");
        setup.finish_setup();
        #[cfg(debug_assertions)]
        crate::client::trace::record(&rt);
    }

    /// One turn (design §4「turn」): every queued event, then everything
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
        host.turns.set(host.turns.get() + 1);
        let turn = rt.begin_turn(events.len());
        turn.run_mounts(kivact_types::Revision(before));
        turn.deliver_reports();
        turn.stabilize();
        let edits = TextEditsAhead::count(&rt, events);
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
            if self.embed_closing() {
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
        let mut writes = 0;
        let mut round = 0;
        loop {
            if crate::client::dispatch::run_requested_activations(&rt) {
                turn.stabilize();
                turn.settle_async();
            }
            writes += commit(&turn, &rt, &tree);
            let mounts = rt.mounts_due(done);
            if !mounts && !rt.pending().user {
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

        let output = output(turn.finish(writes, requests::has_activations(&rt)));
        #[cfg(debug_assertions)]
        {
            crate::client::trace::record(&rt);
            crate::client::trace::event(&rt, format_args!("host_events output {output:?}"));
        }
        output
    }
}
fn output(report: TurnReport) -> TurnOutput {
    TurnOutput {
        next_wake_ns: report.next_deadline,
        exit: matches!(report.exit, ExitState::Exited),
    }
}

impl Drop for AppClient {
    fn drop(&mut self) {
        let Launch::Started(started) = &self.launch else {
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
        #[cfg(feature = "embed")]
        if self.embed_closing.is_some() && kivact_types::process::DetachedThreads::any() {
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
        E::HoverChanged { node } => format!("HoverChanged {:?}", node.map(|n| n.0)),
        E::MousePosition(position) => format!("MousePosition {position:?}"),
        E::TextChanged { node, .. } => format!("TextChanged n{}", node.0),
        E::SelectionChanged { node, .. } => format!("SelectionChanged n{}", node.0),
        E::Ime { node, .. } => format!("Ime n{}", node.0),
        E::ScrollChanged { node, .. } => format!("ScrollChanged n{}", node.0),
        E::LayoutChanged { node, .. } => format!("LayoutChanged n{}", node.0),
        E::A11y { node, request, .. } => format!("A11y {:?} n{}", request.action(), node.0),
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
    if ViewState::tree_mut(rt).take_removes() {
        // Removed nodes report nothing more: forget their failures,
        // so the set stays as small as the live tree.
        let tree = ViewState::tree(rt);
        HostState::of(rt)
            .asset_failures
            .borrow_mut()
            .retain(|(id, _)| tree.contains(*id));
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
