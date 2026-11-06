//! Role: The one in-flight run of latest-wins work (a resource fetch, a `Latest` run).

use std::cell::Cell;
use std::future::Future;
use std::panic::Location;
use std::rc::Rc;

use super::state::TaskCell;
use crate::{Owned, Owner, Runtime};
use kivact_types::cell::Generation;

/// The run in flight: its owner task and its id.
#[derive(Clone, Copy)]
enum Latest {
    Idle,
    Running {
        task: Owned<Rc<TaskCell>>,
        generation: Generation,
    },
}

/// At most one run in flight; starting one cancels the previous one, and a run publishes only
/// while it is still the current one (a `start` or `cancel` during its last poll stops it too).
pub(crate) struct LatestRun {
    current: Cell<Latest>,
    generation: Cell<Generation>,
}

impl LatestRun {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(LatestRun {
            current: Cell::new(Latest::Idle),
            generation: Cell::new(Generation::new()),
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
        let mut generation = self.generation.get();
        let id = generation.next();
        self.generation.set(generation);
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
        self.current.set(Latest::Running {
            task,
            generation: id,
        });
    }

    /// Cancel the run in flight; returns whether there was one.
    pub(crate) fn cancel(&self) -> bool {
        match self.current.replace(Latest::Idle) {
            Latest::Running { task, .. } => {
                task.remove();
                true
            }
            Latest::Idle => false,
        }
    }

    /// Whether a run is in flight.
    pub(crate) fn is_running(&self) -> bool {
        matches!(self.current.get(), Latest::Running { .. })
    }

    /// Run `id` completed: whether it is still the current one (it then is no longer in flight).
    fn end(&self, id: Generation) -> bool {
        let current = matches!(self.current.get(), Latest::Running { generation, .. } if generation.is_current(id));
        if current {
            self.current.set(Latest::Idle);
        }
        current
    }
}
