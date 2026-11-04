//! Role: Defines portable shell configuration, errors and wake handles.

use std::fmt;
use std::sync::Arc;

/// Environmental error crossing a shell callback boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellError {
    /// An available platform adapter failed, with its existing category.
    Failed(kivact_types::Failure),
    /// The platform has no implementation of this service.
    Unsupported(String),
}
impl ShellError {
    /// Construct a backend failure with diagnostic context.
    pub fn new(message: impl Into<String>) -> Self {
        Self::Failed(kivact_types::Failure::new(
            kivact_types::FailureKind::Backend,
            message,
        ))
    }
    /// The platform has no such service.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }
    /// Whether the platform has no such service.
    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported(_))
    }
}
impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(failure) => failure.detail.fmt(f),
            Self::Unsupported(message) => message.fmt(f),
        }
    }
}
impl std::error::Error for ShellError {}

/// Initial window options in logical pixels.
#[derive(Clone, Debug)]
pub struct ShellOptions {
    /// Initial title.
    pub title: String,
    /// Initial content width.
    pub width: f64,
    /// Initial content height.
    pub height: f64,
}
impl Default for ShellOptions {
    fn default() -> Self {
        Self {
            title: "Arvane".into(),
            width: kivact_types::defaults::DEFAULT_WINDOW_SIZE.width,
            height: kivact_types::defaults::DEFAULT_WINDOW_SIZE.height,
        }
    }
}
/// What the event loop does after a callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellControl {
    /// Sleep until OS input or a [`ShellWaker`] wake.
    Wait,
    /// Deliver `ShellEvent::Wake` at this deadline (shell monotonic nanoseconds) unless something else
    /// runs first.
    WakeAt(u64),
    /// Leave the event loop after this callback. An embedded child cannot exit by itself: it reports a
    /// failure.
    Exit,
}
impl ShellControl {
    /// The wake deadline, for [`Self::WakeAt`].
    pub fn deadline(self) -> Option<u64> {
        match self {
            Self::WakeAt(ns) => Some(ns),
            Self::Wait | Self::Exit => None,
        }
    }
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
