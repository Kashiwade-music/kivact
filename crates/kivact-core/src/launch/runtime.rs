//! Role: Concrete host callbacks and the App's start (the turn's order is
//! in `turn`).

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::api::{Client, HostWaker, Writer};
use kivact_engine::contract::{HostInfo, TurnOutput};
use kivact_types::{WindowId, WindowMode};

use crate::client::state::HostState;
use crate::platform::{AppHandle, AppHandleExt};

use crate::view::View;
use crate::view::requests;
use crate::view::state::{ViewState, WindowDecl};
use kivact_runtime::{
    AppParts, ExitState, IntoBinding, IntoValue, RecordBits, Runtime, RuntimeConfig, Stage,
    ThreadWaker, TurnGuard, TurnReport,
};

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
    /// The run's cross-thread handle; `Drop` ends it.
    handle: AppHandle,
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

/// Set up and mount an App whose runtime is built.
fn start_mount(
    rt: &Rc<Runtime>,
    writer: &Writer,
    mut options: AppOptions,
    root: Box<dyn FnOnce() -> View>,
    at: &'static Location<'static>,
) {
    let setup = rt.begin_setup();
    let root_owner = rt.root();
    writer.set_window(
        WindowId::MAIN,
        std::mem::take(&mut options.settings.title),
        options.settings.min_window_size,
        true,
    );
    // The App's fonts precede the root's mount (the engine registers
    // a font when it is written, so the first layout uses them).
    requests::register_app_fonts(rt, std::mem::take(&mut options.settings.fonts));
    let families = ViewState::tree(rt).reader().font_families();
    rt.enter(|| ViewState::of(rt).fonts_value.set(families));
    crate::view::root::install_app_contexts(rt, root_owner, options.into_view_settings(), at);
    crate::view::root::mount_window_root(rt, WindowId::MAIN, root_owner, root, at);
    // What the platform cannot do is diagnosed, not silently ignored:
    // `tracing` says it at start, and the turn trace when the App
    // relies on it (`platform::safe_area`).
    let missing = crate::platform::missing_capabilities(&ViewState::of(rt).capabilities());
    if !missing.is_empty() {
        crate::client::notice::inform(&format!("the platform lacks: {}", missing.join(", ")));
    }
    setup.stabilize();
    let written = commit(&setup, rt, writer);
    debug_assert!(written > 0, "the first revision sets the window");
    setup.finish_setup();
    rt.enter(|| ViewState::of(rt).initial_render.set(false));
    #[cfg(feature = "devtools")]
    crate::devtools::start::after_mount(rt);
}

/// The recorders a launch turns on from the start: a test screen records its run.
fn record_bits(mode: &ClientMode) -> RecordBits {
    if matches!(mode, ClientMode::Test { .. }) {
        RecordBits::TEST
    } else {
        RecordBits::default()
    }
}

/// The comparison trace records the turns when it is on.
#[cfg(debug_assertions)]
fn trace_recorder(rt: &Runtime) {
    if crate::client::trace::Trace::of(rt).is_on() {
        rt.set_recorder(RecordBits::TRACE_SINK, true);
    }
}

impl Launch {
    fn start(&mut self, info: &HostInfo, waker: HostWaker, writer: Writer) {
        let (options, _) = match &mut self.launch {
            LaunchState::Waiting { options, root } => (options, root),
            LaunchState::Started(_) => unreachable!("invariant: host starts an App once"),
        };
        #[cfg(debug_assertions)]
        let trace_name = match &self.mode {
            ClientMode::Test { name, .. } => Some(name.clone()),
            _ => None,
        };
        let handle = options.handle();
        let ready = waker.clone();
        let thread_waker = ThreadWaker::new(move || ready.wake());
        // Bind the handle before the root mounts, so `close` from setup (or
        // from another thread) starts the exit on the next turn.
        handle.attach(thread_waker.clone());
        let at = self.at;
        #[cfg(feature = "devtools")]
        let mut slot = Some(crate::devtools::start::agent_slot(
            &options.settings.title,
            self.mode
                .devtools_allowed(options.settings.devtools_in_tests),
            !matches!(self.mode, ClientMode::Embedded(_)),
            thread_waker.clone(),
        ));
        let rt = Runtime::new(
            Rc::new(ClientHost(waker)),
            thread_waker,
            RuntimeConfig {
                start_ns: info.now_ns,
                blocking_threads: options.settings.blocking_threads,
                at,
                record: record_bits(&self.mode),
            },
            |_rt, root| {
                #[cfg(feature = "devtools")]
                let devtools = crate::devtools::state::DevtoolsState::from_slot(
                    root,
                    slot.take().expect("invariant: the build runs once"),
                    at,
                );
                #[cfg(feature = "devtools")]
                let hooks = devtools.hooks();
                #[cfg(not(feature = "devtools"))]
                let hooks = None;
                let view = ViewState::new(root, info, writer.clone(), hooks, at);
                let motion_env = install_main_window(&view, root, info, options, at);
                let state = crate::app_state::AppState {
                    view,
                    gpu: crate::gpu_interop::StateSlot::default(),
                    handle: AppHandleExt(handle.clone()),
                    #[cfg(feature = "devtools")]
                    devtools,
                    #[cfg(debug_assertions)]
                    trace: crate::client::trace::Trace::new(trace_name.as_deref()),
                    lifecycle: crate::platform::lifecycle::LifecycleState::default(),
                    close: crate::platform::close_request::CloseRequestState::default(),
                    host: HostState::default(),
                    activate: crate::client::dispatch::run_activation,
                };
                AppParts {
                    state: Box::new(state),
                    motion_env,
                }
            },
        );
        #[cfg(debug_assertions)]
        trace_recorder(&rt);
        let (options, root) = match std::mem::replace(
            &mut self.launch,
            LaunchState::Started(Started {
                rt: rt.clone(),
                handle,
            }),
        ) {
            LaunchState::Waiting { options, root } => (*options, root),
            LaunchState::Started(_) => unreachable!("invariant: host starts an App once"),
        };
        #[cfg(debug_assertions)]
        crate::client::trace::event(&rt, format_args!("host_events start {info:?}"));
        start_mount(&rt, &writer, options, root, at);
        #[cfg(debug_assertions)]
        crate::client::trace::record(&rt);
    }
}
/// Install the main window's record and build the App motion clock from it.
fn install_main_window(
    view: &ViewState,
    root: kivact_runtime::Owner,
    info: &HostInfo,
    options: &AppOptions,
    at: &'static Location<'static>,
) -> kivact_runtime::MotionEnv {
    let decl = Rc::new(WindowDecl {
        title: root.signal(options.settings.title.clone(), at).into_value(),
        open: root.signal(true, at).into_binding(),
        size: options
            .settings
            .window_size
            .unwrap_or(kivact_types::defaults::DEFAULT_WINDOW_SIZE),
        min_size: options.settings.min_window_size,
        mode: WindowMode::Normal,
        placement: kivact_types::WindowPlacement::Auto,
        dismiss: None,
        foreign: None,
    });
    let record = crate::view::window_view::open_window_record(
        WindowId::MAIN,
        root,
        root,
        decl,
        info.env.clone(),
        info.surface_active,
        at,
    );
    let env = record.env.clone();
    view.install_window(WindowId::MAIN, record);
    let app_motion = view.app_motion;
    kivact_runtime::MotionEnv {
        reduced_motion: env.reduced_motion_value(),
        surface_active: root.with_setup(|| kivact_runtime::memo(move || app_motion.get().drawable)),
        frame_period_ns: root
            .with_setup(|| kivact_runtime::memo(move || app_motion.get().frame_period_ns)),
    }
}

pub(super) fn output(report: TurnReport) -> TurnOutput {
    if matches!(report.exit, ExitState::Exited) {
        return TurnOutput::Exit;
    }
    TurnOutput::Continue {
        next_wake_ns: report.next_deadline,
    }
}

impl Drop for Launch {
    fn drop(&mut self) {
        // A handle never outlives its run: later `close` calls do nothing.
        match &self.launch {
            LaunchState::Started(started) => started.handle.end(),
            LaunchState::Waiting { options, .. } => options.handle().end(),
        }
        let LaunchState::Started(started) = &self.launch else {
            return;
        };
        let rt = &started.rt;
        let tasks = rt.tasks();
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

pub(super) fn commit(turn: &TurnGuard<'_>, rt: &Runtime, writer: &Writer) -> usize {
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
    /// A test launch: its timing reporter and the test's name (the trace
    /// file is named after it).
    Test {
        timing: TestTiming,
        name: String,
    },
}
/// Reports the duration of each core turn to the test harness.
pub struct TestTiming(pub Rc<dyn Fn(u64)>);

impl ClientMode {
    /// Whether the DevTools agent may run for this launch.
    #[cfg(feature = "devtools")]
    fn devtools_allowed(&self, in_tests: bool) -> bool {
        match self {
            ClientMode::Window | ClientMode::Embedded(_) => true,
            ClientMode::Test { .. } => in_tests,
        }
    }
}

pub(crate) fn run_native(
    client: Client,
    options: kivact_engine::api::HostOptions,
) -> crate::Result<()> {
    crate::platform::run_native(client, options)
}
