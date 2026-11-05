//! Role: Lifecycle-bound resources for components.
//!
//! `Resource<T>` (Solid `createResource`): refetch when the source changes,
//! drop the previous future, accept only the latest request.

use std::future::Future;
use std::panic::Location;
use std::rc::Rc;

use super::async_state::{AsyncCell, AsyncState, cancelled};
use super::latest::LatestRun;
use crate::{EffectKind, IntoValue, Notifier, Owned, Owner, Runtime, UiError, require_effectful};

pub(crate) struct ResourceCell<T: 'static> {
    async_state: AsyncCell<T>,
    refetch: Notifier,
    fetch: Rc<LatestRun>,
}

/// Async data driven by a reactive source; made by [`resource`].
pub struct Resource<T: 'static> {
    handle: Owned<Rc<ResourceCell<T>>>,
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
        async_state: AsyncCell::new(owner, at),
        refetch: owner.notifier(at),
        fetch: LatestRun::new(),
    });
    let handle = owner.store("Resource", cell.clone(), at);
    let weak = Rc::downgrade(&cell);
    owner.effect(
        EffectKind::Internal,
        move || {
            let Some(rt) = Runtime::get(app) else { return };
            // The source only computes a value: pure.
            let s = crate::pure(|| source.get());
            let Some(cell) = weak.upgrade() else { return };
            cell.refetch.track();
            crate::untrack(|| {
                let publish_to = Rc::downgrade(&cell);
                cell.fetch.start(
                    &rt,
                    owner,
                    at,
                    || {
                        cell.async_state.start();
                        fetch(s)
                    },
                    move |r| {
                        let Some(cell) = publish_to.upgrade() else {
                            return;
                        };
                        cell.async_state.finish(r);
                    },
                );
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
        self.cell(Location::caller())
            .async_state
            .state
            .get()
            .as_ref()
            .clone()
    }

    pub(super) fn snapshot(&self) -> super::async_publication::AsyncSnapshot<T> {
        super::async_publication::AsyncSnapshot(
            self.cell(Location::caller()).async_state.state.get(),
        )
    }

    /// Last successful value, kept while refetching, after errors and
    /// after cancellation (tracked).
    #[track_caller]
    pub fn latest(&self) -> Option<T> {
        self.cell(Location::caller()).async_state.latest.get()
    }

    /// Error of the latest request while it is `Failed` (a cancellation
    /// included), `None` otherwise (tracked).
    #[track_caller]
    pub fn error(&self) -> Option<UiError> {
        self.cell(Location::caller())
            .async_state
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
        if cell.fetch.cancel() {
            let err = cancelled(at);
            cell.async_state.finish(Err(err));
        }
    }
}
