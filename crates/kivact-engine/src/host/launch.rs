//! Role: Create the host for a client and run it in a native window.

use super::client::Client;
use super::options::HostOptions;
use super::pump::{Host, LastFrame, Phase};
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

/// Assemble the host shared by native windows and the test driver.
pub(crate) fn new(
    client: Client,
    window_size: Size,
    engine: arvane::Engine,
    frame_log: Rc<RefCell<FrameLog>>,
) -> Host {
    Host {
        client,
        // Replaced by a worker-thread store once the shell's waker
        // exists (`start`); nothing loads before.
        model: Rc::new(RefCell::new(Model::new(
            window_size,
            arvane::asset::AssetStore::inline(),
        ))),
        engine: Rc::new(RefCell::new(engine)),
        last_frame: LastFrame::Settled,
        next_wake: None,
        embedded_closing: None,
        environment: kivact_types::Environment::default(),
        reduced_motion_override: None,
        suspended: false,
        soft_keyboard: false,
        frame_log,
        phase: Phase::Starting,
        adapter: None,
    }
}
