//! Role: Start the DevTools agent and turn its configured chords into engine keys.

use kivact_devtools_agent::{Agent, AgentConfig};
use kivact_devtools_protocol as p;
use kivact_engine::contract::{DevtoolsKeys, KeyChord};
use kivact_runtime::ThreadWaker;

use crate::devtools::convert;
use crate::view::devtools_state::AgentSlot;

/// Start the agent when the launch and the environment allow it.
pub(crate) fn agent_slot(
    title: &str,
    allowed: bool,
    can_inject: bool,
    waker: ThreadWaker,
) -> AgentSlot {
    if !allowed || std::env::var("KIVACT_DEVTOOLS").as_deref() == Ok("0") || is_devtools_app() {
        return AgentSlot::Off;
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
    match Agent::start(AgentConfig { app }, Box::new(move || waker.wake())) {
        Ok(agent) => {
            if !cfg!(debug_assertions) {
                tracing::warn!(
                    target: "kivact::devtools",
                    "the DevTools agent is enabled in a release build"
                );
            }
            AgentSlot::Running { agent, can_inject }
        }
        Err(error) => {
            tracing::warn!(target: "kivact::devtools", "cannot start the DevTools agent: {error}");
            AgentSlot::Off
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
