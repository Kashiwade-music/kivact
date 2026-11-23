//! Role: Per-App extensions of the layers above the graph, and the turn's work they do.
//!
//! The reactive runtime does not know tasks or motion. The host layer
//! installs their state once, when the App is created (`Runtime::new`), as
//! extensions keyed by type; the owning module reads it back with its own
//! accessor (`TaskState::of`, `MotionState::of`). A turn reaches the task
//! subsystem only through [`TurnWork`].

use std::rc::Rc;

use crate::reactive::data::Runtime;

/// The work a turn asks of the task subsystem: whether spawned futures are
/// ready, whether other threads sent mail, and the delivery of that mail at
/// the start of a turn.
pub(crate) trait TurnWork {
    /// Whether a spawned future is ready to be polled (the next turn has
    /// tasks to run).
    fn ready_work(&self) -> bool;
    /// Whether other threads sent mail that the next turn delivers.
    fn mail_waiting(&self) -> bool;
    /// Deliver every mailbox once, as host code of `rt`.
    fn deliver_mail(&self, rt: &Runtime);
}

impl Runtime {
    /// The extension of type `T` of this App. It is installed by
    /// `Runtime::new` before any user code runs; a missing one is a bug of the
    /// host layer.
    pub(crate) fn extension<T: 'static>(&self) -> &T {
        self.extensions
            .get::<T>()
            .expect("invariant: Runtime::new installs every extension before the App runs")
    }

    /// The task subsystem's work for the turn.
    pub(crate) fn work(&self) -> &dyn TurnWork {
        self.extension::<Rc<dyn TurnWork>>().as_ref()
    }
}

/// The extensions of a test runtime that runs no tasks.
#[cfg(test)]
pub(crate) struct NoTasks;

#[cfg(test)]
impl TurnWork for NoTasks {
    fn ready_work(&self) -> bool {
        false
    }
    fn mail_waiting(&self) -> bool {
        false
    }
    fn deliver_mail(&self, _rt: &Runtime) {}
}

/// Extensions for a test runtime that runs no tasks (the task work only).
#[cfg(test)]
pub(crate) fn test_extensions() -> crate::reactive::data::Extensions {
    crate::reactive::data::Extensions::new().with(Rc::new(NoTasks) as Rc<dyn TurnWork>)
}
