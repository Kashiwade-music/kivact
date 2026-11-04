//! Role: Sealed resource and action adapters for standard async views.
use super::{Action, AsyncState, Resource};
use std::rc::Rc;

/// Retry the operation represented by an async view.
#[derive(Clone)]
pub struct Retry(Rc<dyn Fn()>);
impl Retry {
    /// Run the source again with its most recent input.
    pub fn run(&self) {
        (self.0)();
    }
}
/// One canonical publication of an async source, compared by identity.
// xtask-allow(internal-contract): Standard async views compare canonical publications across crate boundaries.
#[doc(hidden)]
pub struct AsyncSnapshot<T>(pub(super) Rc<AsyncState<T>>);
impl<T> Clone for AsyncSnapshot<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> PartialEq for AsyncSnapshot<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: Clone> AsyncSnapshot<T> {
    /// Read the state belonging to this publication.
    pub fn state(&self) -> AsyncState<T> {
        self.0.as_ref().clone()
    }
}
/// A resource or action that can be displayed by an async view.
#[diagnostic::on_unimplemented(
    message = "async_view takes a Resource or Action",
    note = "pass the source handle directly; do not call state()"
)]
pub trait AsyncSource<T>: super::sealed::AsyncSourceSeal + Copy + 'static {
    /// Read the current tracked state.
    fn state(&self) -> AsyncState<T>;
    // xtask-allow(internal-contract): Standard async views consume the sealed source publication identity.
    #[doc(hidden)]
    fn snapshot(&self) -> AsyncSnapshot<T>;
    /// Obtain a callback that retries this source.
    fn retry(&self) -> Retry;
}
impl<T: Clone + 'static> super::sealed::AsyncSourceSeal for Resource<T> {}
impl<T: Clone + 'static> AsyncSource<T> for Resource<T> {
    fn state(&self) -> AsyncState<T> {
        Resource::state(self)
    }
    fn snapshot(&self) -> AsyncSnapshot<T> {
        self.snapshot()
    }
    fn retry(&self) -> Retry {
        let source = *self;
        Retry(Rc::new(move || source.refetch()))
    }
}
impl<T: Clone + 'static, I: 'static> super::sealed::AsyncSourceSeal for Action<T, I> {}
impl<T: Clone + 'static, I: 'static> AsyncSource<T> for Action<T, I> {
    fn state(&self) -> AsyncState<T> {
        Action::state(self)
    }
    fn snapshot(&self) -> AsyncSnapshot<T> {
        self.snapshot()
    }
    fn retry(&self) -> Retry {
        let source = *self;
        Retry(Rc::new(move || source.retry()))
    }
}
