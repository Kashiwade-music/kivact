//! Role: Concrete client callbacks and the reactive turn loop.
//!
//! turn(events, boundaries), the one entry after `start` (design §4「turn」):
//!   0. `AfterCommit` (on_mount of the revisions whose boundaries were
//!      delivered before), `NextTurn` (errors kept from the last layout,
//!      `Runtime::report_later`, are routed to their handlers)
//!   1. each event: resolve node (stale → dropped), run handler untracked,
//!      `AfterHandler` (requested activations), stabilise (Render/Internal
//!      effects write directly to the engine); at each commit boundary,
//!      `AfterCommit` of its revision
//!   2. timers, ready tasks, stabilise (≤ 1000 rounds, then a Cycle
//!      diagnostic and the rest in the next turn)
//!   3. `AfterHandler`, then `phases::commit` (frame request, focus
//!      requests, removed nodes, the commit of the engine's pending changes
//!      and the mount token; revision + 1, nothing if empty)
//!   4. `AfterCommit` of laid-out revisions, user effects; their changes
//!      loop back to step 3 for a further revision (≤ 100 rounds, then a
//!      Cycle diagnostic)
//!   5. next wake = earliest timer / drain deadline; frames via
//!      `Writer::request_frame`
//!
//! What each phase drains, in which order and why there is
//! `client::phases`.
//!
//! Every turn runs these steps: the events of commit boundaries (focus
//! results ..) are committed, with their effects, in their own turn too.
//! Effects write directly to the owned `Writer` the client got at start.
//! Which events a turn carries is described in `crate::client`.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;
use std::sync::Arc;

use kivact_engine::api::{Boundary, Client, HostWaker, Writer};
use kivact_engine::contract::{HostEvent, HostInfo, TurnOutput};
use kivact_types::Size;

use crate::client::phases::{self, Phase};
use crate::client::settle::settle_async;
use crate::client::state::{ExitPhase, HostState};
use crate::i18n::Messages;
use crate::view::View;
use crate::view::requests;
use crate::view::root::AppSettings;
use crate::view::state::ViewState;
use crate::view::text_edits::TextEditsAhead;
use kivact_runtime::MotionState;
use kivact_runtime::TaskState;
use kivact_runtime::{ErrorKind, Extensions, Runtime, TurnRecorder, UiError};
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
    Waiting(Box<AppConfig>),
    /// The host started the App.
    Started,
}

/// The App side of the host boundary: owns the App's runtime and implements
/// concrete callbacks for the engine window loop or test driver.
pub(crate) struct AppClient {
    rt: Rc<Runtime>,
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
        let host = HostState::default();
        let driver = host.link.clone();
        let extensions = Extensions::new().with(ViewState::new());
        #[cfg(feature = "gpu-interop")]
        let extensions = extensions.with(crate::gpu_interop::State::default());
        #[cfg(debug_assertions)]
        let extensions = extensions.with(crate::client::trace::Trace::default());
        let rt = Runtime::new(
            extensions
                .with(TaskState::new(threads))
                .with(MotionState::default())
                .with(crate::platform::lifecycle::LifecycleState::default())
                .with(host),
            driver,
        );
        AppClient {
            rt,
            at,
            launch: Launch::Waiting(Box::new(config)),
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
    #[cfg(feature = "test")]
    pub(crate) fn runtime(&self) -> &Rc<Runtime> {
        &self.rt
    }

    /// Adapt the runtime owner to the engine's concrete callback bundle.
    pub(crate) fn into_client(self, timing: Option<Rc<dyn Fn(u64)>>) -> Client {
        let shared = Rc::new(RefCell::new(self));
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

    /// Deliver `events` in order; a revision's `on_mount` callbacks run
    /// once the events before its boundary ran.
    fn dispatch_all(&self, turn: &TurnRecorder<'_>, events: &[HostEvent], boundaries: &[Boundary]) {
        let edits = TextEditsAhead::count(&self.rt, events);
        let mut boundaries = boundaries.iter().peekable();
        let mut acknowledge = |delivered: usize| {
            while let Some(b) = boundaries.next_if(|b| b.events <= delivered) {
                phases::drain(
                    &self.rt,
                    Phase::AfterCommit {
                        revision: b.revision.0,
                    },
                );
                self.rt.stabilize();
            }
        };
        acknowledge(0);
        let mut delivered = 0;
        for e in events {
            if self.embed_closing() {
                break;
            }
            #[cfg(debug_assertions)]
            crate::client::trace::event(&self.rt, format_args!("host_events receive {e:?}"));
            edits.deliver(e);
            turn.cause(describe(e));
            crate::client::dispatch::dispatch(&self.rt, e);
            phases::drain(&self.rt, Phase::AfterHandler { settle: None });
            self.rt.stabilize();
            delivered += 1;
            acknowledge(delivered);
        }
        // Every boundary lies within the batch: those after an embedder's
        // close are released too.
        acknowledge(events.len());
    }
}

impl AppClient {
    fn start(&mut self, info: &HostInfo, waker: HostWaker, writer: Writer) {
        let config = match std::mem::replace(&mut self.launch, Launch::Started) {
            Launch::Waiting(config) => *config,
            Launch::Started => unreachable!("invariant: the host starts an App once"),
        };
        let rt = self.rt.clone();
        let ready = waker.clone();
        TaskState::of(&rt).set_waker(Arc::new(move || ready.wake()));
        HostState::of(&rt).link.start(info.now_ns, waker);
        let root_owner = rt.create_root(self.at);
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
        writer.set_window(config.title, config.min_window_size);
        // The App's fonts precede the root's mount (the engine registers
        // a font when it is written, so the first layout uses them).
        requests::register_app_fonts(&rt, config.fonts);
        crate::view::root::mount_root(&rt, root_owner, config.settings, config.root, self.at);
        // What the platform cannot do is diagnosed, not silently ignored:
        // `tracing` says it at start, and the turn trace when the App
        // relies on it (`platform::safe_area`).
        let missing = crate::platform::missing_capabilities(&ViewState::of(&rt).capabilities());
        if !missing.is_empty() {
            super::notice::inform(&format!("the platform lacks: {}", missing.join(", ")));
        }
        rt.stabilize();
        let written = phases::commit(&rt, &writer);
        debug_assert!(written > 0, "the first revision sets the window");
        rt.finish_setup_notes();
        #[cfg(debug_assertions)]
        crate::client::trace::record(&rt);
        let pending = rt.pending().user || TaskState::of(&rt).has_ready();
        rt.clear_turn_request();
        if pending {
            rt.request_turn();
        }
    }

    /// One turn (design §4「turn」): every queued event, then everything
    /// they cause (tasks and timers, activations, commits, `on_mount` and
    /// user effects) until nothing is left, the writes committed. The
    /// boundaries among the events only decide when the `on_mount`
    /// callbacks of their revisions run.
    fn turn(&mut self, events: &[HostEvent], boundaries: &[Boundary], now_ns: u64) -> TurnOutput {
        let rt = self.rt.clone();
        let host = HostState::of(&rt);
        host.advance_clock(now_ns);
        let tree = ViewState::tree(&rt).engine().clone();
        // Commit boundaries that carry no event and release no callback
        // make no turn.
        if events.is_empty()
            && boundaries
                .last()
                .is_some_and(|b| !rt.mounts().due(b.revision.0))
        {
            return self.output(&rt);
        }
        // The revisions done before the events: all but those whose
        // boundaries this turn delivers.
        let before = boundaries
            .first()
            .map_or(tree.stats().revision, |b| b.revision.0.saturating_sub(1));
        rt.clear_turn_request();
        let in_turn = host.link.enter_turn();
        host.turns.set(host.turns.get() + 1);
        let turn = rt.turn(events.len());
        #[cfg(debug_assertions)]
        crate::client::trace::stage(&rt, "Begin");
        phases::drain(&rt, Phase::AfterCommit { revision: before });
        // Errors the engine's last layout raised (a primitive's measure
        // function) reach their handlers first, outside any engine borrow.
        phases::drain(&rt, Phase::NextTurn);
        rt.stabilize();
        self.dispatch_all(&turn, events, boundaries);
        // Every revision so far is laid out and its events delivered.
        let done = tree.stats().revision;
        TaskState::of(&rt).check_draining(&rt);
        settle_async(&rt, &turn);
        let mut writes = 0;
        let mut round = 0;
        loop {
            phases::drain(
                &rt,
                Phase::AfterHandler {
                    settle: Some(&turn),
                },
            );
            writes += phases::commit(&rt, &tree);
            // A setup that wrote nothing waits for a revision laid out
            // already: its callbacks run in this turn.
            let mounts = rt.mounts().due(done);
            if !mounts && !rt.pending().user {
                break;
            }
            round += 1;
            if round > USER_EFFECT_ROUNDS {
                let abandoned = rt.abandon_user_effects();
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
                phases::drain(&rt, Phase::AfterCommit { revision: done });
                rt.stabilize();
            }
            rt.run_user_effects();
            rt.stabilize();
            settle_async(&rt, &turn);
        }

        let output = self.output(&rt);
        #[cfg(debug_assertions)]
        crate::client::trace::stage(&rt, "End");
        turn.finish(writes);
        #[cfg(debug_assertions)]
        {
            crate::client::trace::record(&rt);
            crate::client::trace::event(&rt, format_args!("host_events output {output:?}"));
        }
        // Requests made during the turn woke nobody (`HostLink`): ask again
        // only for work the turn left.
        drop(in_turn);
        let pending = rt.pending().user
            || TaskState::of(&rt).has_ready()
            || rt.pending().later
            || requests::has_activations(&rt);
        rt.clear_turn_request();
        if pending {
            rt.request_turn();
        }
        output
    }

    /// Whether the App may exit now (a requested close whose queued
    /// actions drained, or whose drain deadline passed) and when the host
    /// must wake next.
    fn output(&self, rt: &Runtime) -> TurnOutput {
        let host = HostState::of(rt);
        let tasks = TaskState::of(rt);
        let phase = match host.exit.get() {
            ExitPhase::Draining { .. } if !tasks.queues_busy() => ExitPhase::Exited,
            ExitPhase::Draining { deadline_ns } if rt.now_ns() >= deadline_ns => {
                let n = tasks.abandon_all_queues();
                rt.report(
                    None,
                    UiError::new(
                        ErrorKind::Closed,
                        format!("exit drain deadline passed; {n} queued run(s) abandoned"),
                        None,
                    ),
                );
                ExitPhase::Exited
            }
            phase => phase,
        };
        host.exit.set(phase);
        let next_wake_ns = match phase {
            ExitPhase::Draining { deadline_ns } => Some(
                tasks
                    .next_deadline()
                    .map_or(deadline_ns, |t| t.min(deadline_ns)),
            ),
            ExitPhase::Running | ExitPhase::Exited => tasks.next_deadline(),
        };
        TurnOutput {
            next_wake_ns,
            exit: phase == ExitPhase::Exited,
        }
    }
}

impl Drop for AppClient {
    fn drop(&mut self) {
        let tasks = TaskState::of(&self.rt);
        // A propagating panic ends the App: run no more App code. Only the
        // worker threads are stopped.
        if !std::thread::panicking() {
            if let Some(root) = self.rt.root() {
                root.dispose();
            }
            tasks.abandon_all_queues();
            // async-task schedules cancellation when its handle is dropped.
            // Release those futures on the UI thread before the executor
            // stops; their channel wakers can otherwise retain its queue and
            // receivers.
            if tasks.has_ready() {
                tasks.poll_tasks(&self.rt);
            }
        }
        tasks.shutdown();
        if !std::thread::panicking() {
            self.rt.shutdown();
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
