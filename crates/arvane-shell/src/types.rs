//! Role: Defines portable shell configuration, errors and wake handles.

use std::fmt;
use std::sync::Arc;

/// Environmental error crossing a shell callback boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellError {
    message: String,
    /// The platform has no such service (rather than the service failing).
    unsupported: bool,
}
impl ShellError {
    /// Construct an error with diagnostic context.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            unsupported: false,
        }
    }
    /// An error saying the platform has no such service at all (an
    /// operation Arvane does not implement there), as opposed to one that
    /// failed this time.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            unsupported: true,
        }
    }
    /// Whether the platform has no such service ([`ShellError::unsupported`]).
    pub fn is_unsupported(&self) -> bool {
        self.unsupported
    }
}
impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
impl std::error::Error for ShellError {}

/// Initial window options in logical pixels.
#[derive(Clone, Debug)]
pub struct ShellOptions {
    /// Initial title.
    pub title: String,
    /// Initial content width.
    pub width: f32,
    /// Initial content height.
    pub height: f32,
}
impl Default for ShellOptions {
    fn default() -> Self {
        Self {
            title: "Arvane".into(),
            width: 960.0,
            height: 720.0,
        }
    }
}
/// Callback control; absent deadlines put the loop to sleep until OS input.
#[derive(Clone, Copy, Debug, Default)]
pub struct ShellControl {
    /// Exit after the callback.
    pub exit: bool,
    /// Next callback deadline in shell monotonic nanoseconds.
    pub next_wake_ns: Option<u64>,
}
/// Thread-safe worker wake handle. It owns no window state.
#[derive(Clone)]
pub struct ShellWaker(Arc<dyn Fn() + Send + Sync>);
impl ShellWaker {
    /// Create a thread-safe wake handle for a backend's event loop.
    /// The callback must tolerate a loop that has already closed.
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Arc::new(wake))
    }

    /// Schedule a Wake event; a closed loop ignores further wakes.
    pub fn wake(&self) {
        (self.0)();
    }
}
impl std::fmt::Debug for ShellWaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShellWaker")
    }
}
