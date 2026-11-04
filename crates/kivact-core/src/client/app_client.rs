//! Role: Concrete host callbacks and the ordered App turn loop.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::{Boundary, Client, HostWaker, Writer};
use kivact_engine::contract::{HostEvent, HostInfo, TurnOutput};
use kivact_types::Size;

use crate::client::state::HostState;
use crate::i18n::{Locale, Messages};
use crate::view::View;
use crate::view::requests;
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

/// An application: title, window size, theme, fonts and the root
/// component.
#[derive(Clone)]
pub struct AppOptions {
    title: String,
    theme: Theme,
    /// `None`: the system's locale (`App::locale`).
    locale: Option<Locale>,
    messages: Messages,
    /// The window's first size (`window_size`); the engine's default when
    /// `None`.
    window_size: Option<Size>,
    /// The window's minimum size (`min_window_size`).
    min_window_size: Option<Size>,
    blocking_threads: Option<usize>,
    fonts: Vec<(Asset, &'static Location<'static>)>,
    /// How the window's GPU device is made (`gpu_preference`).
    gpu_preference: kivact_types::GpuPreference,
}

impl AppOptions {
    /// An app whose window is titled `title`, with the default theme.
    pub fn new(title: impl Into<String>) -> AppOptions {
        AppOptions {
            title: title.into(),
            theme: Theme::default(),
            locale: None,
            messages: Messages::new(),
            window_size: None,
            min_window_size: None,
            blocking_threads: None,
            fonts: Vec::new(),
            gpu_preference: kivact_types::GpuPreference::default(),
        }
    }

    /// Theme of the whole App (to switch it at runtime, wrap the root in
    /// `view::provide_theme` with a `Signal<Theme>`).
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// The App's locale: the language and region components format for.
    /// Without it the App speaks the system's locale (`en-US` where the
    /// system names none) and follows it when the user changes it while
    /// the App runs; with it the App keeps this locale whatever the system
    /// says. To switch it at runtime, wrap the root in
    /// [`view::provide_locale`](crate::view::provide_locale) with a `Signal<Locale>`. Read it with
    /// [`i18n::locale`](crate::i18n::locale).
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }

    /// The App's texts: the replacements for the default (English) texts
    /// of components, by key (`"dialog.close"`). Each component lists its
    /// keys. To switch them at runtime, wrap the root in
    /// [`view::provide_messages`](crate::view::provide_messages) with a `Signal<Messages>`.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.messages = messages;
        self
    }

    /// The window's first content size in logical pixels (default 960 by
    /// 720), e.g. `.window_size(1280, 800)`. The user can resize the
    /// window afterwards; read the current size with
    /// [`platform::window_size`](crate::platform::window_size) (to save it for the next start, say).
    /// Both values must be finite, at least one logical pixel, fit the native
    /// window extent, and not be smaller than
    /// [`crate::App::min_window_size`]: otherwise [`crate::App::run`] returns an error
    /// of kind `ErrorKind::Message` without opening the window.
    pub fn window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.window_size = Some(Size::new(width.into(), height.into()));
        self
    }

    /// The smallest content size in logical pixels the user can resize the
    /// window to, e.g. `.min_window_size(480, 320)` (default: no minimum).
    /// Both values must be finite, at least one logical pixel, fit the native
    /// window extent, and not be larger than
    /// [`crate::App::window_size`] when that is set: otherwise [`crate::App::run`]
    /// returns an error of kind `ErrorKind::Message` without opening the
    /// window.
    pub fn min_window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.min_window_size = Some(Size::new(width.into(), height.into()));
        self
    }

    /// Register the fonts of a TTF, OTF or TTC file before the first
    /// screen, so text can name their families with
    /// `style().font_family(..)` from the start (like CSS `@font-face`).
    /// Call it once per file; a family is the name written in the font,
    /// not the file name. Usually embedded:
    /// `.font(include_bytes!("../assets/Inter-Regular.ttf"))`. A file that
    /// is not a usable font (WOFF, unreadable) is reported as
    /// `ErrorKind::Asset`, located here; the App starts anyway.
    #[track_caller]
    pub fn font(mut self, font: impl Into<Asset>) -> Self {
        self.fonts.push((font.into(), Location::caller()));
        self
    }

    /// What the App wishes of the window's GPU device (experimental,
    /// feature `gpu-interop`): the kind of adapter, the graphics API and
    /// the adapter itself (default: Arvane's own choice). A wish: the
    /// device is made on another adapter or API when the one asked for is
    /// not there, and the environment variables `WGPU_BACKEND` and
    /// `WGPU_POWER_PREF` override it. Read the adapter the device is on
    /// with [`gpu_interop::adapter`](crate::gpu_interop::adapter).
    #[cfg(feature = "gpu-interop")]
    pub fn gpu_preference(mut self, preference: crate::gpu_interop::GpuPreference) -> Self {
        self.gpu_preference = preference;
        self
    }

    /// Size of the blocking pool (default `min(4, CPUs)`).
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.blocking_threads = Some(n.max(1));
        self
    }

    #[track_caller]
    pub fn check_window_sizes(&self) -> crate::Result<()> {
        crate::app::check_window_sizes(self.window_size, self.min_window_size, Location::caller())
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    /// Initial engine settings, derived from the application options.
    pub fn host_options(&self) -> kivact_engine::api::HostOptions {
        #[cfg(feature = "gpu-interop")]
        let gpu_preference = self.gpu_preference;
        #[cfg(not(feature = "gpu-interop"))]
        let gpu_preference = kivact_types::GpuPreference::default();
        kivact_engine::api::HostOptions {
            window_size: self.window_size,
            gpu_preference,
        }
    }
    /// The single title used for both engine startup and later window effects.
    pub fn window_title(&self) -> &str {
        &self.title
    }
    pub(crate) fn into_view_settings(self) -> (Theme, Option<crate::i18n::Locale>, Messages) {
        (self.theme, self.locale, self.messages)
    }
}

/// Whether the host started the App.
enum LaunchState {
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
struct Started {
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
    launch: LaunchState,
    /// Host mode and its close/timing protocol for this application.
    mode: ClientMode,
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

    /// The App's runtime (the Testing Library reads the tree and the
    /// extensions through it).
    fn runtime(&self) -> &Rc<Runtime> {
        match &self.launch {
            LaunchState::Started(started) => &started.rt,
            LaunchState::Waiting { .. } => {
                panic!("invariant: runtime is available after host start")
            }
        }
    }
    fn shared_client(shared: Rc<RefCell<Self>>, timing: Option<Rc<dyn Fn(u64)>>) -> Client {
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
                blocking_threads: options.blocking_threads,
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
                    reduced_motion: env.field("reduced_motion", |e| e.reduced_motion),
                    surface_active: kivact_runtime::Value::from(env.surface_active),
                    frame_period_ns: env.field("frame_period_ns", |e| e.frame_period_ns),
                };
                let view = ViewState::new(
                    root,
                    info,
                    writer.clone(),
                    env,
                    options.min_window_size,
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
        writer.set_window(std::mem::take(&mut options.title), options.min_window_size);
        // The App's fonts precede the root's mount (the engine registers
        // a font when it is written, so the first layout uses them).
        requests::register_app_fonts(&rt, std::mem::take(&mut options.fonts));
        let (options, root) = match std::mem::replace(
            &mut self.launch,
            LaunchState::Started(Started { rt: rt.clone() }),
        ) {
            LaunchState::Waiting { options, root } => (*options, root),
            LaunchState::Started(_) => unreachable!("invariant: host starts an App once"),
        };
        crate::view::root::mount_root(&rt, root_owner, options, root, self.at);
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
        let mut stats = host.stats.get();
        stats.turns += 1;
        host.stats.set(stats);
        let turn = rt.begin_turn(events.len());
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
    let removed = std::mem::take(&mut *host.removed.borrow_mut());
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
/// Read-only handle shared with an engine client.
pub struct LaunchHandle(Rc<RefCell<Launch>>);
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
