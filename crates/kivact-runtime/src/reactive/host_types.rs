//! Role: Host wake contracts and passive turn status values.

/// The host wake boundary of an App.
pub trait Host: 'static {
    /// Request a turn soon on the Quiet to WakeSent edge.
    fn wake(&self);
}
/// A wake usable from any thread.
#[derive(Clone)]
pub struct ThreadWaker(std::sync::Arc<dyn Fn() + Send + Sync>);
impl ThreadWaker {
    /// Create a wake callback usable from any thread.
    pub fn new(f: impl Fn() + Send + Sync + 'static) -> Self {
        Self(std::sync::Arc::new(f))
    }
    /// Invoke the wake callback.
    pub fn wake(&self) {
        (self.0)();
    }
}
/// The operation currently performed in a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// App setup.
    Setup,
    /// Turn entry.
    Begin,
    /// Deferred error reports.
    Reports,
    /// Host event delivery.
    Dispatch,
    /// Tasks, timers and drains.
    Async,
    /// Engine write commit.
    Commit,
    /// Acknowledged mount callbacks.
    Mounts,
    /// User effect execution.
    UserEffects,
    /// Turn output and scheduling.
    Output,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TurnState {
    Quiet,
    WakeSent,
    Running(Stage),
}
/// The App's exit drain status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitState {
    /// No exit was requested.
    Running,
    /// Queued actions drain until the deadline in nanoseconds.
    Draining {
        /// Drain deadline in nanoseconds.
        deadline: u64,
    },
    /// The host may exit.
    Exited,
}
/// Scheduling result of a completed turn.
pub struct TurnReport {
    /// Earliest timer or drain deadline in nanoseconds.
    pub next_deadline: Option<u64>,
    /// Current exit status.
    pub exit: ExitState,
}
