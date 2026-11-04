//! Role: `AsyncState<T>`: the one state shape of resources and actions.

use crate::{ErrorKind, UiError};
use std::panic::Location;

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

/// The only publication path for resource and action state.
pub(crate) struct AsyncCell<T: 'static> {
    pub(super) state: crate::Signal<AsyncState<T>>,
    pub(super) latest: crate::Signal<Option<T>>,
}
impl<T: Clone + 'static> AsyncCell<T> {
    pub(super) fn new(owner: crate::Owner, at: &'static Location<'static>) -> Self {
        let cell = Self {
            state: owner.signal(AsyncState::Idle, at),
            latest: owner.signal(None, at),
        };
        cell.reset();
        cell
    }
    pub(super) fn start(&self) {
        self.state.update(|state| *state = AsyncState::Pending);
    }
    pub(super) fn finish(&self, result: Result<T, UiError>) {
        let next = match result {
            Ok(value) => {
                self.latest.update(|latest| *latest = Some(value.clone()));
                AsyncState::Ready(value)
            }
            Err(error) => AsyncState::Failed(error),
        };
        self.state.update(|state| *state = next);
    }
    pub(super) fn reset(&self) {
        self.state.update(|state| *state = AsyncState::Idle);
    }
}
