//! Role: Display the loading, success and retry states of async operations.
use kivact_core::UiError;
use kivact_core::task::{AsyncSnapshot, AsyncSource, AsyncState, Retry};
use kivact_core::view::{IntoView, NodeBuilder, View, column, dynamic, text};
use std::rc::Rc;

/// A view of a resource or action, with optional loading and error content.
pub struct AsyncView<T> {
    state: Rc<dyn Fn() -> AsyncSnapshot<T>>,
    retry: Retry,
    ready: Rc<dyn Fn(T) -> View>,
    loading: Rc<dyn Fn() -> View>,
    fallback: Rc<dyn Fn(UiError, Retry) -> View>,
}
/// Display an async source. Failures include a retry button by default.
pub fn async_view<T: Clone + 'static, V: IntoView>(
    source: impl AsyncSource<T>,
    ready: impl Fn(T) -> V + 'static,
) -> AsyncView<T> {
    AsyncView {
        state: Rc::new(move || source.snapshot()),
        retry: source.retry(),
        ready: Rc::new(move |value| ready(value).into_view()),
        loading: Rc::new(default_loading),
        fallback: Rc::new(default_fallback),
    }
}
impl<T> AsyncView<T> {
    /// Replace the default loading indicator.
    pub fn loading<V: IntoView>(mut self, f: impl Fn() -> V + 'static) -> Self {
        self.loading = Rc::new(move || f().into_view());
        self
    }
    /// Replace the default error message and retry button.
    pub fn fallback<V: IntoView>(mut self, f: impl Fn(UiError, Retry) -> V + 'static) -> Self {
        self.fallback = Rc::new(move |error, retry| f(error, retry).into_view());
        self
    }
}
impl<T: Clone + 'static> IntoView for AsyncView<T> {
    fn into_view(self) -> View {
        dynamic(
            move || (self.state)(),
            move |snapshot| match snapshot.state() {
                AsyncState::Idle => ().into_view(),
                AsyncState::Pending => (self.loading)(),
                AsyncState::Ready(value) => (self.ready)(value),
                AsyncState::Failed(error) => (self.fallback)(error, self.retry.clone()),
            },
        )
    }
}
fn default_loading() -> View {
    column((
        super::progress(None::<f64>).name("Loading"),
        text("Loading…"),
    ))
    .into_view()
}
fn default_fallback(error: UiError, retry: Retry) -> View {
    column((
        text(format!("Error: {error}")),
        super::button("Retry").on_press(move || retry.run()),
    ))
    .into_view()
}
