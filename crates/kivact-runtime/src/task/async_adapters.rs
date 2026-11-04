//! Role: Adapt resources and actions to the async source contract.
use super::{Action, AsyncSnapshot, AsyncSource, AsyncState, Resource, Retry};
use std::rc::Rc;

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
