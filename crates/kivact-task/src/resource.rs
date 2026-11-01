//! Role: Lifecycle-bound resources for components.
//!
//! `Resource<T>` (Solid `createResource`): refetch when the source changes,
//! drop the previous future, accept only the latest request.

use std::cell::Cell;
use std::future::Future;
use std::panic::Location;
use std::rc::Rc;

use super::async_state::{AsyncState, cancelled};
use super::executor::catch_unwind;
use super::state::TaskCell;
use kivact_reactive::{
    EffectKind, IntoValue, Owned, Owner, Runtime, Signal, Trigger, UiError, require_effectful,
};

pub(crate) struct ResourceCell<T: 'static> {
    state: Signal<AsyncState<T>>,
    latest: Signal<Option<T>>,
    refetch: Trigger,
    task: Cell<Option<Owned<TaskCell>>>,
    generation: Cell<u64>,
}

/// Async data driven by a reactive source; made by [`resource`].
pub struct Resource<T: 'static> {
    handle: Owned<ResourceCell<T>>,
}

impl<T: 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for Resource<T> {}

/// Fetch `fetch(source)` now and whenever `source` changes (Solid
/// `createResource`). `fetch` and its future run untracked: only `source`
/// (and `refetch`) is a dependency. `source` is anything a property takes: a `Signal`, a
/// `Value`, a closure such as `move || (id.get(), page.get())`, or a
/// constant. Only the latest request's result is accepted. `fetch` returns
/// `Result<T, UiError>`; `?` converts any other `std::error::Error`. Owned
/// by the current setup owner; setup only.
///
/// ```ignore
/// let user = resource(id, |id| async move { load(id).await });
/// ```
#[track_caller]
pub fn resource<T, S, Fut>(
    source: impl IntoValue<S>,
    fetch: impl Fn(S) -> Fut + 'static,
) -> Resource<T>
where
    T: Clone + 'static,
    S: Clone + 'static,
    Fut: Future<Output = Result<T, UiError>> + 'static,
{
    let at = Location::caller();
    let source = source.into_prop();
    let owner = Owner::current("resource", at);
    let app = owner.app();
    let cell = Rc::new(ResourceCell {
        state: owner.signal(AsyncState::<T>::Idle, at),
        latest: owner.signal(None::<T>, at),
        refetch: owner.trigger(at),
        task: Cell::new(None),
        generation: Cell::new(0),
    });
    let handle = owner.store("Resource", cell.clone(), at);
    let weak = Rc::downgrade(&cell);
    owner.effect(
        EffectKind::Internal,
        move || {
            let Some(rt) = Runtime::get(app) else { return };
            // The source only computes a value: pure.
            let s = kivact_reactive::pure(|| source.get());
            let Some(cell) = weak.upgrade() else { return };
            cell.refetch.track();
            kivact_reactive::untrack(|| {
                let generation = cell.generation.get() + 1;
                cell.generation.set(generation);
                if let Some(t) = cell.task.take() {
                    t.remove();
                }
                cell.state.update(|st| *st = AsyncState::Pending);
                // A panic before the future exists fails this request
                // instead of leaving the state `Pending`.
                let fut = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fetch(s)))
                {
                    Ok(fut) => fut,
                    Err(p) => {
                        let err = UiError::from_panic(p, Some(at));
                        cell.state.update(move |st| *st = AsyncState::Failed(err));
                        return;
                    }
                };
                let weak = Rc::downgrade(&cell);
                let task = super::spawn::spawn_owned(
                    &rt,
                    owner,
                    async move {
                        let r = catch_unwind(fut).await;
                        let Some(cell) = weak.upgrade() else { return };
                        if cell.generation.get() != generation {
                            return;
                        }
                        cell.task.set(None);
                        let next = match r {
                            Ok(Ok(v)) => {
                                let latest = v.clone();
                                cell.latest.update(move |l| *l = Some(latest));
                                AsyncState::Ready(v)
                            }
                            Ok(Err(e)) => AsyncState::Failed(e),
                            Err(p) => AsyncState::Failed(UiError::from_panic(p, Some(at))),
                        };
                        cell.state.update(move |st| *st = next);
                    },
                    at,
                );
                cell.task.set(Some(task));
            });
        },
        at,
    );
    Resource { handle }
}

impl<T: Clone + 'static> Resource<T> {
    fn cell(&self, at: &'static Location<'static>) -> Rc<ResourceCell<T>> {
        self.handle.get_at(at)
    }

    /// State of the latest request (tracked). A cancelled request is
    /// `Failed` with an error whose `is_cancelled()` is true.
    #[track_caller]
    pub fn state(&self) -> AsyncState<T> {
        self.cell(Location::caller()).state.get()
    }

    /// Last successful value, kept while refetching, after errors and
    /// after cancellation (tracked).
    #[track_caller]
    pub fn latest(&self) -> Option<T> {
        self.cell(Location::caller()).latest.get()
    }

    /// Error of the latest request while it is `Failed` (a cancellation
    /// included), `None` otherwise (tracked).
    #[track_caller]
    pub fn error(&self) -> Option<UiError> {
        self.cell(Location::caller())
            .state
            .with(|state| state.error().cloned())
    }

    /// Fetch again with the current source value.
    #[track_caller]
    pub fn refetch(&self) {
        self.cell(Location::caller()).refetch.notify();
    }

    /// Drop the in-flight request: the state becomes `Failed` with a
    /// cancellation error. A no-op when nothing is in flight.
    #[track_caller]
    pub fn cancel(&self) {
        let at = Location::caller();
        let cell = self.cell(at);
        // Checked before any state changes.
        require_effectful("Resource::cancel");
        if let Some(t) = cell.task.take() {
            cell.generation.set(cell.generation.get() + 1);
            t.remove();
            let err = cancelled(at);
            cell.state.update(move |st| *st = AsyncState::Failed(err));
        }
    }
}
