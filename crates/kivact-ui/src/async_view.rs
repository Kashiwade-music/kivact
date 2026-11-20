//! Role: Display the loading, success and retry states of async operations.
use kivact_core::UiError;
use kivact_core::state::{Callback, untrack};
use kivact_core::task::{AsyncSource, AsyncState};
use kivact_core::view::{Element, IntoView, IntoViewFn, View, dynamic};
use std::rc::Rc;

/// A view of a resource or action, with optional loading and error content.
pub struct AsyncView<T> {
    publications: Rc<dyn Fn() -> u64>,
    state: Rc<dyn Fn() -> AsyncState<T>>,
    retry: Callback<()>,
    ready: Rc<dyn Fn(T) -> View>,
    loading: Rc<dyn Fn() -> View>,
    fallback: Rc<dyn Fn(UiError, Callback<()>) -> View>,
}
/// Display an async source. Failures include a retry button by default.
/// The button is named by the message `async_view.retry` ("Retry").
pub fn async_view<T: Clone + 'static, V: IntoView>(
    source: impl AsyncSource<T>,
    ready: impl Fn(T) -> V + 'static,
) -> AsyncView<T> {
    AsyncView {
        publications: Rc::new(move || source.publications()),
        state: Rc::new(move || source.state()),
        retry: source.retry(),
        ready: Rc::new(move |value| ready(value).into_view()),
        loading: Rc::new(default_loading),
        fallback: Rc::new(default_fallback),
    }
}
impl<T> AsyncView<T> {
    /// Replace the default loading indicator.
    pub fn loading<V: IntoView>(mut self, f: impl IntoViewFn<V>) -> Self {
        let f = f.into_view_fn();
        self.loading = Rc::new(move || f.call(()));
        self
    }
    /// Replace the default error message and retry button.
    pub fn fallback<V: IntoView>(
        mut self,
        f: impl Fn(UiError, Callback<()>) -> V + 'static,
    ) -> Self {
        self.fallback = Rc::new(move |error, retry| f(error, retry).into_view());
        self
    }
}
impl<T: Clone + 'static> IntoView for AsyncView<T> {
    fn into_view(self) -> View {
        dynamic(
            move || (self.publications)(),
            move |_| match untrack(|| (self.state)()) {
                AsyncState::Idle => ().into_view(),
                AsyncState::Pending => (self.loading)(),
                AsyncState::Ready(value) => (self.ready)(value),
                AsyncState::Failed(error) => (self.fallback)(error, self.retry),
            },
        )
        .into_view()
    }
}
fn default_loading() -> View {
    super::spinner().into_view()
}
fn default_fallback(error: UiError, retry: Callback<()>) -> View {
    super::alert(format!("Error: {error}"))
        .tone(kivact_interaction::Tone::Danger)
        .actions(
            super::button(kivact_core::i18n::message("async_view.retry", "Retry")).on_press(retry),
        )
        .into_view()
}
