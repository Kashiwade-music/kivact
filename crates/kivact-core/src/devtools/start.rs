//! Role: Start the DevTools agent and turn its configured chords into engine keys.

use kivact_devtools_agent::{Agent, AgentConfig};
use kivact_devtools_protocol as p;
use kivact_engine::contract::{DevtoolsKeys, KeyChord};
use kivact_runtime::{Runtime, ThreadWaker};

use crate::devtools::convert;
use crate::devtools::state::{AgentSlot, AgentStart, Inspector};
use crate::view::state::ViewState;

/// Start the agent on a background thread when the launch and the
/// environment allow it; `None` otherwise.
pub(crate) fn agent_slot(
    title: &str,
    allowed: bool,
    can_inject: bool,
    waker: ThreadWaker,
) -> Option<AgentStart> {
    if !allowed || std::env::var("KIVACT_DEVTOOLS").as_deref() == Ok("0") || is_devtools_app() {
        return None;
    }
    let app = p::AppInfo {
        name: title.to_owned(),
        pid: u64::from(std::process::id()),
        kivact_version: env!("CARGO_PKG_VERSION").to_owned(),
        build: if cfg!(debug_assertions) {
            p::BuildKind::Debug
        } else {
            p::BuildKind::Release
        },
        exe: std::env::current_exe()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        cwd: std::env::current_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        workspace_root: String::new(),
    };
    let (sender, receiver) = std::sync::mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("kivact-devtools-start".to_owned())
        .spawn(move || {
            let wake = waker.clone();
            let started = Agent::start(AgentConfig { app }, Box::new(move || wake.wake()));
            let _ = sender.send(started);
            waker.wake();
        });
    match spawned {
        Ok(_) => Some(AgentStart {
            receiver,
            can_inject,
        }),
        Err(error) => {
            tracing::warn!(target: "kivact::devtools", "cannot start the DevTools agent: {error}");
            None
        }
    }
}

/// Whether this process is the DevTools app itself.
fn is_devtools_app() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|stem| stem == "kivact-devtools"))
        .unwrap_or(false)
}

/// The engine keys of the configured chords; a chord that cannot be read is
/// dropped with a warning.
pub(crate) fn engine_keys(config: &p::DevtoolsConfig) -> DevtoolsKeys {
    DevtoolsKeys {
        open: chords(&config.open_keys()),
        pick: chords(&config.pick_keys()),
    }
}

fn chords(values: &[String]) -> Vec<KeyChord> {
    let mut chords = Vec::new();
    for value in values {
        match convert::parse_chord(value) {
            Some(chord) => chords.push(chord),
            None => tracing::warn!(
                target: "kivact::devtools",
                "ignoring an unreadable DevTools key chord: {value:?}"
            ),
        }
    }
    chords
}

/// The DevTools work after the App's root mounted: the agent is promoted if it
/// has started, and the engine learns the configured key chords.
pub(crate) fn after_mount(rt: &kivact_runtime::Runtime) {
    if let crate::devtools::state::DevtoolsState::Running(i) =
        crate::devtools::state::DevtoolsState::of(rt)
    {
        i.promote(rt);
    }
}

impl Inspector {
    /// Move a started agent into the ready slot. The first turn after the agent
    /// starts, or the mount, does this; a failed start is warned about once.
    pub(crate) fn promote(&self, rt: &Runtime) {
        let started = match &*self.agent.borrow() {
            AgentSlot::Starting(receiver) => match receiver.try_recv() {
                Ok(started) => started,
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Err(std::io::Error::other("the DevTools start thread ended"))
                }
            },
            AgentSlot::Ready(_) | AgentSlot::Failed => return,
        };
        let next = match started {
            Ok(agent) => {
                if !cfg!(debug_assertions) {
                    tracing::warn!(
                        target: "kivact::devtools",
                        "the DevTools agent is enabled in a release build"
                    );
                }
                let keys = engine_keys(agent.config());
                let tree = ViewState::tree(rt);
                tree.engine().set_devtools_keys(keys);
                tree.reader().record_tree_changes(true);
                drop(tree);
                AgentSlot::Ready(Box::new(agent))
            }
            Err(error) => {
                tracing::warn!(target: "kivact::devtools", "cannot start the DevTools agent: {error}");
                AgentSlot::Failed
            }
        };
        *self.agent.borrow_mut() = next;
    }
}
