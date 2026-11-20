//! Role: Adapt resources and actions to the async source contract.
use super::{Action, AsyncSource, AsyncState, Resource};

impl<T: Clone + 'static> super::sealed::AsyncSourceSeal for Resource<T> {}
impl<T: Clone + 'static> AsyncSource<T> for Resource<T> {
    fn state(&self) -> AsyncState<T> {
        Resource::state(self)
    }
    fn retry(&self) -> crate::Callback<()> {
        let source = *self;
        crate::Callback::from_handler(crate::Handler::from_fn(move |()| source.refetch()))
    }
    fn publications(&self) -> u64 {
        Resource::publications(self)
    }
}
impl<T: Clone + 'static, I: 'static> super::sealed::AsyncSourceSeal for Action<T, I> {}
impl<T: Clone + 'static, I: 'static> AsyncSource<T> for Action<T, I> {
    fn state(&self) -> AsyncState<T> {
        Action::state(self)
    }
    fn retry(&self) -> crate::Callback<()> {
        let source = *self;
        crate::Callback::from_handler(crate::Handler::from_fn(move |()| source.retry()))
    }
    fn publications(&self) -> u64 {
        Action::publications(self)
    }
}
