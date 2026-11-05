//! Role: Passive async publication identity and retry callbacks.
use super::AsyncState;
use std::rc::Rc;

/// Retry a failed operation: the source of an async view (again with its
/// most recent input), or the child of an error boundary (mounted fresh).
#[derive(Clone)]
pub struct Retry(pub(super) Rc<dyn Fn()>);
impl Retry {
    /// A retry that runs `f`.
    pub fn new(f: impl Fn() + 'static) -> Self {
        Self(Rc::new(f))
    }
    /// Try the failed operation again.
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
