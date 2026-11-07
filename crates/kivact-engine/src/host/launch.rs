//! Role: Create the host for a client and run it in a native window.

use super::client::Client;
use super::options::HostOptions;
use super::pump::{Host, HostPhase, LastPresent};
use crate::FrameLog;
use crate::contract::HostError;
use crate::model::Model;
use arvane::{RendererChange, Shell, ShellOptions};
use kivact_types::Size;
use std::cell::RefCell;
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
        arvane_native::backend(ShellOptions {
            width: window_size.width,
            height: window_size.height,
            title: "Kivact".into(),
        })
        .map_err(|e| HostError::new(e.to_string()))?,
    );
    let shell = shell.gpu_preference(options.gpu_preference);
    let shell = shell.on_renderer_change(move |change| {
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
impl arvane::Handler for Launch {
    fn start(
        &mut self,
        window: &mut arvane::Window<'_>,
        waker: arvane::ShellWaker,
    ) -> Result<arvane::ShellControl, arvane::ShellError> {
        let Self::Pending(args) = std::mem::replace(self, Self::Starting) else {
            panic!("invariant: shell starts its handler once");
        };
        let args = *args;
        let store = match args.assets {
            AssetMode::Inline => arvane::asset::AssetStore::inline(),
            AssetMode::Worker => {
                let wake = waker.clone();
                arvane::asset::AssetStore::new(move || wake.wake(arvane::WakeReasons::ASSETS))
            }
        };
        let mut model = Model::new(args.window_size, store);
        model.frame.start_theme(args.theme);
        let client = args.client;
        let engine = args.engine;
        let mut host = Host {
            client,
            model: Rc::new(RefCell::new(model)),
            engine: Rc::new(RefCell::new(engine)),
            phase: HostPhase::Starting,
            presence: super::pump::AppPresence::Foreground,
            surface: super::pump::Surface::Unavailable(super::pump::Unavailable::NoSurface),
            last_present: LastPresent::Settled,
            next_wake: None,
            close: args.close.clone(),
            environment: kivact_types::cell::Sent::new(),
            reduced_motion_override: None,
            soft_keyboard: kivact_types::cell::Sent::new(),
            ime_allowed: kivact_types::cell::Sent::new(),
            frame_log: args.frame_log.clone(),
            accessibility_noted: kivact_types::cell::ReportOnce::new(),
            adapter: kivact_types::cell::Sent::new(),
            last_stats: None,
        };
        let result = host.begin(
            window,
            super::client::HostWaker::new(move || waker.wake(arvane::WakeReasons::CLIENT)),
        );
        *self = Self::Running(Box::new(host));
        result
    }
    fn event(
        &mut self,
        event: arvane::ShellEvent,
        window: &mut arvane::Window<'_>,
    ) -> Result<arvane::ShellControl, arvane::ShellError> {
        self.host_mut().handle(event, window)
    }
}
