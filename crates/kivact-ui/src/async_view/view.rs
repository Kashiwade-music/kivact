//! Role: The async view component: loading, ready and failed states.

use crate::async_view::look::AsyncViewTheme;
use crate::layer::with_base_recipe_style;
use kivact_core::UiError;
use kivact_core::state::{Callback, untrack};
use kivact_core::task::{AsyncSource, AsyncState};
use kivact_core::view::{Element, IntoView, Variants, View, ViewFn, dynamic, node};
use kivact_macros::component;
use std::rc::Rc;

/// Display an async source: `ready` renders its value, `loading` (by
/// default a spinner) shows while it is pending, and `fallback` (by default
/// an error message with a retry button) shows a failure. The retry button
/// is named by the message `async_view.retry` ("Retry").
///
/// The root is a column that holds the current content; the look is the
/// theme's [`AsyncViewTheme::root`](crate::theme::AsyncViewTheme).
#[component]
pub fn async_view<T: Clone + 'static, S: AsyncSource<T>>(
    source: S,
    ready: Rc<dyn Fn(T) -> View>,
    loading: Option<ViewFn>,
    fallback: Option<Rc<dyn Fn(UiError, Callback<()>) -> View>>,
) -> impl Element {
    let loading = loading.unwrap_or_else(|| ViewFn::from(|()| default_loading()));
    let fallback: Rc<dyn Fn(UiError, Callback<()>) -> View> = match fallback {
        Some(fallback) => fallback,
        None => Rc::new(default_fallback),
    };
    let retry = source.retry();
    let content = dynamic(
        move || source.publications(),
        move |_| match untrack(|| source.state()) {
            AsyncState::Idle => ().into_view(),
            AsyncState::Pending => loading.call(()),
            AsyncState::Ready(value) => ready(value),
            AsyncState::Failed(error) => fallback(error, retry),
        },
    );
    with_base_recipe_style(
        node(),
        |r: &AsyncViewTheme| r.root_recipe(),
        &Variants::default(),
        || (),
    )
    .child(content.into_view())
}

fn default_loading() -> View {
    crate::spinner().into_view()
}

fn default_fallback(error: UiError, retry: Callback<()>) -> View {
    crate::alert(format!("Error: {error}"))
        .tone(kivact_interaction::Tone::Danger)
        .actions(
            crate::button(kivact_core::i18n::message("async_view.retry", "Retry")).on_press(retry),
        )
        .into_view()
}
