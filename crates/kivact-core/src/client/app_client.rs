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
//!   3. `AfterHandler`, then `BeforeCommit` (frame request, focus requests,
//!      removed nodes, the mount token) and the commit of the engine's
//!      pending changes (revision + 1; nothing if empty)
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

use kivact_engine::bridge::Boundary;
use kivact_engine::bridge::{Client, HostWaker, Writer};
use kivact_wire::{HostEvent, HostInfo, Size, TurnOutput};

use crate::client::phases::{self, Phase};
use crate::client::settle::settle_async;
use crate::client::state::HostState;
use crate::i18n::{Locale, Messages};
use crate::view::View;
use crate::view::requests;
use crate::view::state::ViewState;
use kivact_motion::MotionState;
use kivact_reactive::{ErrorKind, Extensions, Runtime, TurnRecorder, UiError};
use kivact_style::Theme;
use kivact_style::asset::Asset;
use kivact_task::TaskState;

/// Rounds of `on_mount` callbacks and user effects one turn runs before
/// the rest is cut as a `Cycle`.
const USER_EFFECT_ROUNDS: usize = 100;

/// Commit directly applied changes as the next revision.
/// Returns the engine's operation count (0: nothing to commit).
fn flush(rt: &Runtime, tree: &Writer) -> usize {
    let count = phases::drain(rt, Phase::BeforeCommit { writer: tree }).writes;
    if count != 0 {
        tree.commit();
    }

    count
}

/// The App side of the host boundary: owns the App's runtime and implements
/// concrete callbacks for the engine window loop or test driver.
pub(crate) struct AppClient {
    rt: Rc<Runtime>,
    root: Option<Box<dyn FnOnce() -> View>>,
    /// Where the App was started: the location of the root owner.
    at: &'static Location<'static>,
    title: String,
    theme: Theme,
    /// `None`: the App speaks the host's locale.
    locale: Option<Locale>,
    messages: Messages,
    #[cfg(feature = "embed")]
    embed_failure: Option<Arc<std::sync::Mutex<Option<UiError>>>>,
    /// Set by `Embedded::drop`: the turn in progress finishes, but the rest
    /// of its event batch is not delivered.
    #[cfg(feature = "embed")]
    embed_closed: Option<Arc<std::sync::atomic::AtomicBool>>,
    /// Fonts registered before the root is mounted (`App::font`).
    fonts: Vec<(Asset, &'static Location<'static>)>,
}

impl AppClient {
    /// An App titled `title` whose root is built by `root` when the host
    /// starts, in a setup frame of the root owner; `threads` sizes the
    /// blocking pool (default `min(4, CPUs)`). `at` is where the App was
    /// started (`App::run`, `test::render`): diagnostics name the root
    /// owner by it.
    pub(crate) fn new(
        title: String,
        theme: Theme,
        threads: Option<usize>,
        root: Box<dyn FnOnce() -> View>,
        at: &'static Location<'static>,
    ) -> Self {
        let host = HostState::default();
        let driver = host.link.clone();
        let extensions = Extensions::new().with(ViewState::new());
        #[cfg(feature = "gpu-interop")]
        let extensions = extensions.with(crate::gpu_interop::State::default());
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
            root: Some(root),
            at,
            title,
            theme,
            locale: None,
            messages: Messages::new(),
            fonts: Vec::new(),
            #[cfg(feature = "embed")]
            embed_failure: None,
            #[cfg(feature = "embed")]
            embed_closed: None,
        }
    }

    #[cfg(feature = "embed")]
    pub(crate) fn with_embed_failure(
        mut self,
        failure: Arc<std::sync::Mutex<Option<UiError>>>,
        closed: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        self.embed_failure = Some(failure);
        self.embed_closed = Some(closed);
        self
    }

    fn embed_closed(&self) -> bool {
        #[cfg(feature = "embed")]
        if let Some(closed) = &self.embed_closed {
            return closed.load(std::sync::atomic::Ordering::Acquire);
        }
        false
    }

    /// Record an embedded failure only after startup or the whole turn settled.
    fn embed_stop(&self) -> Option<String> {
        #[cfg(feature = "embed")]
        if let Some(failure) = &self.embed_failure
            && let Some(error) = self.rt.first_unhandled_panic()
        {
            let mut failure = failure.lock().unwrap_or_else(|e| e.into_inner());
            let original = failure.get_or_insert(error);
            return Some(original.to_string());
        }
        None
    }

    /// Start the App with `locale` (`None`: the host's) and `messages`
    /// (`App::locale`, `App::messages`).
    pub(crate) fn with_i18n(mut self, locale: Option<Locale>, messages: Messages) -> Self {
        self.locale = locale;
        self.messages = messages;
        self
    }

    /// Register `fonts` (each located at its `App::font` call) when the
    /// host starts: their `RegisterFont` effects come first in the first
    /// revision, before the root's mount, so the first layout measures with
    /// them. A rejected font is reported as `ErrorKind::Asset`.
    pub(crate) fn with_fonts(mut self, fonts: Vec<(Asset, &'static Location<'static>)>) -> Self {
        self.fonts = fonts;
        self
    }

    /// Keep the window at least `size` logical pixels (`None`: no
    /// minimum). The size is used as given: `App::run` validated it.
    pub(crate) fn with_min_window_size(self, size: Option<Size>) -> Self {
        ViewState::of(&self.rt).set_min_window_size(size);
        self
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
        ViewState::tree_mut(&self.rt).count_text_edits(events.iter().filter_map(
            |event| match event {
                kivact_wire::HostEvent::TextChanged { node, .. } => Some(*node),
                _ => None,
            },
        ));
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
            if self.embed_closed() {
                break;
            }
            if let kivact_wire::HostEvent::TextChanged { node, .. } = e {
                ViewState::tree_mut(&self.rt).consume_text_edit(*node);
            }
            turn.cause(describe(e));
            crate::client::dispatch::dispatch(&self.rt, e);
            phases::drain(&self.rt, Phase::AfterHandler { settle: None });
            self.rt.stabilize();
            delivered += 1;
            acknowledge(delivered);
        }
        acknowledge(usize::MAX);
        ViewState::tree_mut(&self.rt).clear_text_edits();
    }
}

impl AppClient {
    fn start(&mut self, info: &HostInfo, waker: HostWaker, tree: Writer) -> Option<String> {
        let rt = self.rt.clone();
        let ready = waker.clone();
        TaskState::of(&rt).set_waker(Arc::new(move || ready.wake()));
        HostState::of(&rt).link.start(info.now_ns, waker);
        MotionState::of(&rt).set_frame_period(info.env.frame_period_ns.max(1));
        // Install the owned writer before setup creates detached host nodes.
        ViewState::tree_mut(&rt).set_engine(tree.clone());
        ViewState::tree(&rt)
            .engine()
            .set_window(self.title.clone(), ViewState::of(&rt).min_window_size());
        // The App's fonts precede the root's mount (the engine registers
        // a font when it is written, so the first layout uses them).
        requests::register_app_fonts(&rt, std::mem::take(&mut self.fonts));
        let root_owner = rt.create_root(self.at);
        #[cfg(feature = "gpu-interop")]
        crate::gpu_interop::State::of(&rt).install_adapter(root_owner, self.at);
        crate::view::root::mount_root(
            &rt,
            root_owner,
            info,
            crate::view::root::AppSettings {
                theme: self.theme.clone(),
                locale: self.locale.clone(),
                messages: self.messages.clone(),
            },
            self.root.take(),
            self.at,
        );
        // What the platform cannot do is diagnosed, not silently ignored:
        // `tracing` says it at start, and the turn trace when the App
        // relies on it (`platform::safe_area`).
        let missing = crate::platform::missing_capabilities(&ViewState::of(&rt).capabilities());
        if !missing.is_empty() {
            super::notice::inform(&format!("the platform lacks: {}", missing.join(", ")));
        }
        rt.stabilize();
        let written = flush(&rt, &tree);
        debug_assert!(written > 0, "the first revision sets the window");
        rt.finish_setup_notes();
        let pending = rt.pending().user || TaskState::of(&rt).has_ready();
        rt.clear_turn_request();
        if pending {
            rt.request_turn();
        }
        self.embed_stop()
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
        host.link.set_in_turn(true);
        host.turns.set(host.turns.get() + 1);
        let started = std::time::Instant::now();
        let turn = rt.turn(events.len());
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
            writes += flush(&rt, &tree);
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

        let mut output = self.output(&rt);
        let took = started.elapsed().as_nanos() as u64;
        turn.finish(writes, took);
        // Requests made during the turn woke nobody (`HostLink`): ask again
        // only for work the turn left.
        host.link.set_in_turn(false);
        let pending = rt.pending().user
            || TaskState::of(&rt).has_ready()
            || rt.pending().later
            || requests::has_activations(&rt);
        rt.clear_turn_request();
        if pending {
            rt.request_turn();
        }
        output.stop = self.embed_stop();
        output
    }

    /// Whether the App may exit now (a requested close whose queued
    /// actions drained, or whose drain deadline passed) and when the host
    /// must wake next.
    fn output(&self, rt: &Runtime) -> TurnOutput {
        let host = HostState::of(rt);
        let now = rt.now_ns();
        let mut exit = false;
        let (close, deadline) = (host.close_requested.get(), host.exit_deadline.get());
        if close {
            if !TaskState::of(rt).queues_busy() {
                exit = true;
            } else if deadline.is_some_and(|d| now >= d) {
                let n = TaskState::of(rt).abandon_all_queues();
                rt.report(
                    None,
                    UiError::new(
                        ErrorKind::Closed,
                        format!("exit drain deadline passed; {n} queued run(s) abandoned"),
                        None,
                    ),
                );
                exit = true;
            }
        }
        let next_wake_ns = [
            TaskState::of(rt).next_deadline(),
            if close && !exit { deadline } else { None },
        ]
        .into_iter()
        .flatten()
        .min();
        TurnOutput {
            next_wake_ns,
            exit,
            stop: None,
        }
    }
}

impl Drop for AppClient {
    fn drop(&mut self) {
        if let Some(root) = self.rt.root() {
            root.dispose();
        }
        let tasks = TaskState::of(&self.rt);
        tasks.abandon_all_queues();
        // async-task schedules cancellation when its handle is dropped.
        // Release those futures on the UI thread before the executor stops;
        // their channel wakers can otherwise retain its queue and receivers.
        if tasks.has_ready() {
            tasks.poll_tasks(&self.rt);
        }
        tasks.shutdown();
        self.rt.shutdown();
        // Opening may fail before an Embedded is returned, or its drop may
        // precede deferred callback cleanup. Record task retention here too.
        #[cfg(feature = "embed")]
        if self.embed_failure.is_some() && kivact_task::detached_threads() {
            kivact_engine::embed::retain_module();
        }
    }
}

/// Short cause label of an event for the structured trace (no values).
fn describe(e: &kivact_wire::HostEvent) -> String {
    use kivact_wire::HostEvent as E;
    match e {
        E::Pointer { node, event, .. } => format!("Pointer {:?} n{}", event.phase, node.0),
        E::Wheel { node, .. } => format!("Wheel n{}", node.0),
        E::Key {
            node, key, region, ..
        } => match node {
            Some(n) => format!("Key {key:?} n{} region {:?}", n.0, region.map(|r| r.0)),
            None => format!("Key {key:?}"),
        },
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
        E::TextChanged { node, .. } => format!("TextChanged n{}", node.0),
        E::SelectionChanged { node, .. } => format!("SelectionChanged n{}", node.0),
        E::Ime { node, .. } => format!("Ime n{}", node.0),
        E::ScrollChanged { node, .. } => format!("ScrollChanged n{}", node.0),
        E::LayoutChanged { node, .. } => format!("LayoutChanged n{}", node.0),
        E::A11y { node, action, .. } => format!("A11y {action:?} n{}", node.0),
        other => {
            let s = format!("{other:?}");
            s.split([' ', '{', '('])
                .next()
                .unwrap_or_default()
                .to_string()
        }
    }
}
