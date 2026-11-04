//! Role: Engine client host contracts.
use kivact_types::{Capabilities, Environment};
use std::fmt;

/// What the client is given at start: the window's environment as the
/// host read it (`Window::environment`). Later changes arrive as
/// `HostEvent::EnvChanged`, with the whole `Environment` again.
#[derive(Clone, Debug, PartialEq)]
pub struct HostInfo {
    /// What the window's backend can do (touch, pen, software keyboard,
    /// safe area, memory warnings, IME reconversion, native accessibility,
    /// GPU), reported once at start.
    pub capabilities: Capabilities,
    /// Host clock at start (ns, monotonic, host-defined epoch).
    pub now_ns: u64,
    /// The window's environment at start.
    pub env: Environment,
    pub surface_active: bool,
}

/// Result of one turn. A turn may commit several revisions (user effects
/// that write state produce a follow-up revision within the same turn).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurnOutput {
    /// Host clock time at which the client wants the next turn even without
    /// events (timers). Frame-synchronised work uses
    /// the bridge's `Writer::request_frame`.
    pub next_wake_ns: Option<u64>,
    /// The App finished (close accepted and queues drained).
    pub exit: bool,
}

/// A host operation failure reported to its client.
#[derive(Clone, Debug, PartialEq)]
pub struct HostError {
    /// Human-readable cause of the failed host operation.
    pub message: String,
}

impl HostError {
    /// Construct a failure with a human-readable cause.
    pub fn new(message: impl Into<String>) -> Self {
        HostError {
            message: message.into(),
        }
    }
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HostError {}
