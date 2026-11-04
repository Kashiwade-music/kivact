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
pub struct ShellWaker(Arc<dyn Fn(WakeReasons) + Send + Sync>);
impl ShellWaker {
    /// Create a thread-safe wake handle for a backend's event loop.
    /// The callback must tolerate a loop that has already closed.
    pub fn new(wake: impl Fn(WakeReasons) + Send + Sync + 'static) -> Self {
        Self(Arc::new(wake))
    }

    /// Schedule a Wake event; a closed loop ignores further wakes.
    pub fn wake(&self, reasons: WakeReasons) {
        (self.0)(reasons);
    }
}
impl std::fmt::Debug for ShellWaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShellWaker")
    }
}

/// Reasons coalesced into one event-loop wake.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WakeReasons(u8);
impl WakeReasons {
    /// Application work is ready.
    pub const CLIENT: Self = Self(1);
    /// Asset loading has progressed.
    pub const ASSETS: Self = Self(2);
    /// Assistive technology has queued work.
    pub const A11Y_ACTIONS: Self = Self(4);
    /// The platform environment has changed.
    pub const ENVIRONMENT: Self = Self(8);
    /// GPU initialization has progressed.
    pub const GPU_STEP: Self = Self(16);

    /// Whether every supplied reason is present.
    pub fn contains(self, reasons: Self) -> bool {
        self.0 & reasons.0 == reasons.0
    }
    /// Include the supplied reasons.
    pub fn insert(&mut self, reasons: Self) {
        self.0 |= reasons.0;
    }
    /// Whether no reason is present.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}
