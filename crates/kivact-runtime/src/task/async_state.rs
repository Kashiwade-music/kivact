//! Role: `AsyncState<T>`: the one state shape of resources and actions.

use crate::{ErrorKind, UiError};
use std::panic::Location;
use std::rc::Rc;

/// State of a [`Resource`](super::Resource) request or an
/// [`Action`](super::Action) run.
///
/// A cancelled request or run is `Failed` with an error whose
/// [`UiError::is_cancelled`] is true.
#[derive(Clone, Debug, PartialEq)]
pub enum AsyncState<T> {
    /// Nothing has started yet.
    Idle,
    /// A request or run is in flight.
    Pending,
    /// The latest request or run succeeded.
    Ready(T),
    /// The latest request or run failed or was cancelled.
    Failed(UiError),
}

impl<T> AsyncState<T> {
    /// Whether a request or run is in flight.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Whether the latest request or run succeeded.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Whether the latest request or run failed or was cancelled.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    /// The value of the latest success, `None` in every other state.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    /// The error of the latest failure (a cancellation included), `None` in
    /// every other state.
    pub fn error(&self) -> Option<&UiError> {
        match self {
            Self::Failed(error) => Some(error),
            _ => None,
        }
    }
}

/// The error a cancelled request or run ends with.
pub(crate) fn cancelled(at: &'static Location<'static>) -> UiError {
    UiError::new(ErrorKind::Cancelled, "cancelled", Some(at))
}

/// The run an action's state describes: its input and attempt number.
pub(crate) struct RunInfo<I> {
    pub(crate) input: Rc<I>,
    pub(crate) attempt: u32,
}

impl<I> Clone for RunInfo<I> {
    fn clone(&self) -> Self {
        RunInfo {
            input: Rc::clone(&self.input),
            attempt: self.attempt,
        }
    }
}

/// Where a resource or an action stands (12 章 ST-01). `last` is the last
/// successful value, kept while a new run is pending or failed; a `Ready`
/// phase always has its value, so no combination misses it.
pub(crate) enum AsyncPhase<T> {
    Idle,
    Pending { last: Option<Rc<T>> },
    Ready(Rc<T>),
    Failed { error: UiError, last: Option<Rc<T>> },
}

impl<T> AsyncPhase<T> {
    /// The last successful value, in any phase.
    fn last(&self) -> Option<&Rc<T>> {
        match self {
            AsyncPhase::Idle => None,
            AsyncPhase::Pending { last } | AsyncPhase::Failed { last, .. } => last.as_ref(),
            AsyncPhase::Ready(value) => Some(value),
        }
    }

    /// This phase taken out (left `Idle`), to build the next one from it.
    fn take_last(&mut self) -> Option<Rc<T>> {
        match std::mem::replace(self, AsyncPhase::Idle) {
            AsyncPhase::Idle => None,
            AsyncPhase::Pending { last } | AsyncPhase::Failed { last, .. } => last,
            AsyncPhase::Ready(value) => Some(value),
        }
    }
}

pub(crate) struct AsyncCellState<T, I> {
    pub(crate) phase: AsyncPhase<T>,
    pub(crate) run: Option<RunInfo<I>>,
    pub(crate) waiting: usize,
    pub(crate) publications: u64,
}

/// The only publication path for resource and action state: one signal.
pub(crate) struct AsyncCell<T: 'static, I: 'static> {
    state: crate::Signal<AsyncCellState<T, I>>,
}

impl<T: Clone + 'static, I: 'static> AsyncCell<T, I> {
    pub(super) fn new(owner: crate::Owner, at: &'static Location<'static>) -> Self {
        let initial = AsyncCellState {
            phase: AsyncPhase::Idle,
            run: None,
            waiting: 0,
            publications: 0,
        };
        Self {
            state: owner.signal(initial, at),
        }
    }

    fn publish(&self, f: impl FnOnce(&mut AsyncCellState<T, I>)) {
        self.state.update(|s| {
            f(s);
            s.publications += 1;
        });
    }

    /// A run starts; `waiting` more wait behind it.
    pub(super) fn begin(&self, run: RunInfo<I>, waiting: usize) {
        self.publish(|s| {
            let last = s.phase.take_last();
            s.phase = AsyncPhase::Pending { last };
            s.run = Some(run);
            s.waiting = waiting;
        });
    }

    /// Work is pending (a resource fetch, or runs queued behind one).
    pub(super) fn pending(&self, waiting: usize) {
        self.publish(|s| {
            let last = s.phase.take_last();
            s.phase = AsyncPhase::Pending { last };
            s.waiting = waiting;
        });
    }

    /// A run ended with `result`; `waiting` runs still wait.
    pub(super) fn finish(&self, result: Result<T, UiError>, waiting: usize) {
        self.publish(|s| {
            s.waiting = waiting;
            let last = s.phase.take_last();
            s.phase = match result {
                Ok(value) => AsyncPhase::Ready(Rc::new(value)),
                Err(error) => AsyncPhase::Failed { error, last },
            };
        });
    }

    /// The public state (tracked).
    pub(super) fn state(&self) -> AsyncState<T> {
        self.state.with(|s| match &s.phase {
            AsyncPhase::Idle => AsyncState::Idle,
            AsyncPhase::Pending { .. } => AsyncState::Pending,
            AsyncPhase::Ready(value) => AsyncState::Ready(T::clone(value)),
            AsyncPhase::Failed { error, .. } => AsyncState::Failed(error.clone()),
        })
    }

    pub(super) fn latest(&self) -> Option<T> {
        self.state.with(|s| s.phase.last().map(|v| T::clone(v)))
    }

    pub(super) fn error(&self) -> Option<UiError> {
        self.state.with(|s| match &s.phase {
            AsyncPhase::Failed { error, .. } => Some(error.clone()),
            _ => None,
        })
    }

    pub(super) fn attempt(&self) -> u32 {
        self.state.with(|s| s.run.as_ref().map_or(0, |r| r.attempt))
    }

    pub(super) fn waiting(&self) -> usize {
        self.state.with(|s| s.waiting)
    }

    pub(super) fn run_untracked(&self) -> Option<RunInfo<I>> {
        self.state.with_untracked(|s| s.run.clone())
    }

    /// How many times the state was written (tracked): async views
    /// re-render on it without comparing values.
    pub(super) fn publications(&self) -> u64 {
        self.state.with(|s| s.publications)
    }
}

impl<T: Clone + 'static, I: Clone + 'static> AsyncCell<T, I> {
    pub(super) fn input(&self) -> Option<I> {
        self.state
            .with(|s| s.run.as_ref().map(|r| I::clone(&r.input)))
    }
}
