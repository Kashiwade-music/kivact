//! Role: The one in-flight run of latest-wins work (a resource fetch, a `Latest` run).

use std::cell::Cell;
use std::future::Future;
use std::panic::Location;
use std::rc::Rc;

use super::state::TaskCell;
use kivact_reactive::{Owned, Owner, Runtime};

/// The run in flight: its owner task and its id.
#[derive(Clone, Copy)]
struct Inflight {
    task: Owned<TaskCell>,
    id: u64,
}

/// At most one run in flight; starting one cancels the previous one, and a run publishes only
/// while it is still the current one (a `start` or `cancel` during its last poll stops it too).
pub(crate) struct LatestRun {
    current: Cell<Option<Inflight>>,
    next_id: Cell<u64>,
}

impl LatestRun {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(LatestRun {
            current: Cell::new(None),
            next_id: Cell::new(0),
        })
    }

    /// Cancel the run in flight, then make the new run's future with `make` and spawn it as a task
    /// of `owner`; `publish` gets its output if it is still the current run when it completes.
    pub(crate) fn start<F>(
        self: &Rc<Self>,
        rt: &Runtime,
        owner: Owner,
        at: &'static Location<'static>,
        make: impl FnOnce() -> F,
        publish: impl FnOnce(F::Output) + 'static,
    ) where
        F: Future + 'static,
    {
        self.cancel();
        let fut = make();
        let id = self.next_id.get() + 1;
        self.next_id.set(id);
        let me = Rc::downgrade(self);
        let task = super::spawn::spawn_owned(
            rt,
            owner,
            async move {
                let out = fut.await;
                let Some(me) = me.upgrade() else { return };
                if !me.end(id) {
                    return;
                }
                publish(out);
            },
            at,
        );
        self.current.set(Some(Inflight { task, id }));
    }

    /// Cancel the run in flight; returns whether there was one.
    pub(crate) fn cancel(&self) -> bool {
        match self.current.take() {
            Some(run) => {
                run.task.remove();
                true
            }
            None => false,
        }
    }

    /// Whether a run is in flight.
    pub(crate) fn is_running(&self) -> bool {
        self.current.get().is_some()
    }

    /// Run `id` completed: whether it is still the current one (it then is no longer in flight).
    fn end(&self, id: u64) -> bool {
        let current = self.current.get().is_some_and(|run| run.id == id);
        if current {
            self.current.set(None);
        }
        current
    }
}
