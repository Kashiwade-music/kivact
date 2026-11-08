//! Role: Create the host for a client and run it in a native window.

use super::client::Client;
use super::options::HostOptions;
use super::pump::{Host, HostPhase};
use super::window::HostWindow;
use crate::FrameLog;
use crate::contract::HostError;
use crate::model::{AppModel, EventSink, Model, Shared};
use arvane::{
    EventTarget, Handler, RendererChange, Shell, ShellControl, ShellError, ShellEvent, ShellWaker,
    Windows,
};
use kivact_types::cell::Sent;
use kivact_types::{Size, WindowId, WindowMode, WindowRequest};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Run the client on the application's main thread until its native window exits.
///
/// The caller validates a requested size as finite and positive before starting.
/// Client callbacks run without an outstanding model borrow. Window creation or
/// event-loop failures return the host error for the caller to report.
pub fn run_native(client: Client, options: HostOptions) -> Result<(), HostError> {
    let window_size = options
        .window_size
        .unwrap_or(kivact_types::defaults::DEFAULT_WINDOW_SIZE);
    let frame_log = Rc::new(RefCell::new(FrameLog::from_env()));
    // Arvane's renderer changes, as they happen (benchmarks parse the
    // `renderer=` notes); the first is the renderer the window starts
    // with.
    let log = frame_log.clone();
    let shell = Shell::new(
        arvane_native::backend(WindowRequest::main("Kivact", window_size))
            .map_err(|e| HostError::new(e.to_string()))?,
    );
    let shell = shell.gpu_preference(options.gpu_preference);
    let shell = shell.on_renderer_change(move |_window, change| {
        let mut log = log.borrow_mut();
        match change {
            RendererChange::Start { .. } => log.note(&format!("host=arvane {change}")),
            _ => log.note(&change.to_string()),
        }
    });
    let engine = arvane::Engine::new();
    let handler = new(client, window_size, engine, frame_log);
    shell
        .run(handler)
        .map_err(|e| HostError::new(e.to_string()))
}

/// Assemble startup arguments shared by native windows and tests.
pub(crate) fn new(
    client: Client,
    window_size: Size,
    engine: arvane::Engine,
    frame_log: Rc<RefCell<FrameLog>>,
) -> Launch {
    Launch::Pending(Box::new(LaunchArgs {
        client,
        window_size,
        engine,
        frame_log,
        theme: kivact_types::ThemeHint::Light,
        assets: AssetMode::Worker,
        close: crate::contract::CloseSignal::new(),
    }))
}

pub(super) enum AssetMode {
    Worker,
    #[cfg(feature = "testing")]
    Inline,
}
pub(crate) struct LaunchArgs {
    client: Client,
    window_size: Size,
    engine: arvane::Engine,
    frame_log: Rc<RefCell<FrameLog>>,
    pub(super) theme: kivact_types::ThemeHint,
    pub(super) assets: AssetMode,
    close: crate::contract::CloseSignal,
}
pub(crate) enum Launch {
    Pending(Box<LaunchArgs>),
    Starting,
    Running(Box<Host>),
}
impl Launch {
    pub(crate) fn with_close(mut self, close: crate::contract::CloseSignal) -> Self {
        if let Self::Pending(args) = &mut self {
            args.close = close;
        }
        self
    }
    #[cfg(feature = "testing")]
    pub(super) fn host(&self) -> &Host {
        match self {
            Self::Running(host) => host,
            Self::Pending(_) | Self::Starting => {
                panic!("invariant: host is queried after shell start")
            }
        }
    }
    pub(super) fn host_mut(&mut self) -> &mut Host {
        match self {
            Self::Running(host) => host,
            Self::Pending(_) | Self::Starting => panic!("invariant: events follow shell start"),
        }
    }
}
impl Handler for Launch {
    fn start(
        &mut self,
        windows: &mut Windows<'_>,
        waker: ShellWaker,
    ) -> Result<ShellControl, ShellError> {
        let Self::Pending(args) = std::mem::replace(self, Self::Starting) else {
            panic!("invariant: shell starts its handler once");
        };
        let args = *args;
        let store = match args.assets {
            #[cfg(feature = "testing")]
            AssetMode::Inline => arvane::asset::AssetStore::inline(),
            AssetMode::Worker => {
                let wake = waker.clone();
                arvane::asset::AssetStore::new(move || wake.wake(arvane::WakeReasons::ASSETS))
            }
        };
        let shared = Shared {
            events: EventSink::default(),
            assets: Rc::new(RefCell::new(crate::lower::assets::ArvaneAssets::new(store))),
        };
        let mut model = Model::new(WindowId::MAIN, args.window_size, &shared);
        model.frame.start_theme(args.theme);
        let mut host_windows = BTreeMap::new();
        host_windows.insert(WindowId::MAIN, HostWindow::new(WindowMode::Normal, None));
        let client = args.client;
        let engine = args.engine;
        let mut host = Host {
            client,
            app: Rc::new(RefCell::new(AppModel::new(shared, model))),
            engine: Rc::new(RefCell::new(engine)),
            phase: HostPhase::Starting,
            presence: super::pump::AppPresence::Foreground,
            next_wake: None,
            close: args.close.clone(),
            reduced_motion_override: None,
            frame_log: args.frame_log.clone(),
            accessibility_noted: kivact_types::cell::ReportOnce::new(),
            windows: host_windows,
            app_motion: Sent::new(),
        };
        let result = host.begin(
            windows,
            super::client::HostWaker::new(move || waker.wake(arvane::WakeReasons::CLIENT)),
        );
        *self = Self::Running(Box::new(host));
        result
    }
    fn event(
        &mut self,
        target: EventTarget,
        event: ShellEvent,
        windows: &mut Windows<'_>,
    ) -> Result<ShellControl, ShellError> {
        self.host_mut().handle(target, event, windows)
    }
    fn close(&mut self, _windows: &mut Windows<'_>) {}
}
