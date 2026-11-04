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
/// A resource or action that can be displayed by an async view.
pub trait AsyncSource<T>: super::sealed::AsyncSourceSeal + Copy + 'static {
    /// Read the current tracked state.
    fn state(&self) -> AsyncState<T>;
    /// Obtain a callback that retries this source.
    fn retry(&self) -> Retry;
}
impl<T: Clone + 'static> super::sealed::AsyncSourceSeal for Resource<T> {}
impl<T: Clone + 'static> AsyncSource<T> for Resource<T> {
    fn state(&self) -> AsyncState<T> {
        Resource::state(self)
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
    fn retry(&self) -> Retry {
        let source = *self;
        Retry(Rc::new(move || source.retry()))
    }
}
