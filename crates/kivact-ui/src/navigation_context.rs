//! Role: Share one existing navigation panel's immutable close request and appearance.
use crate::variants::Variants;
use kivact_core::state::{Callback, provide, try_use_context};
use kivact_core::view::{IntoView, View};
#[derive(Clone)]
pub(crate) struct NavigationContext {
    pub(crate) close: Callback<()>,
    pub(crate) variants: Variants,
}
pub(crate) fn current() -> Option<NavigationContext> {
    try_use_context::<NavigationContext>().map(|context| context.get())
}
pub(crate) fn provide_panel<V: IntoView>(
    close: Callback<()>,
    variants: Variants,
    content: impl FnOnce() -> V + 'static,
) -> View {
    provide(NavigationContext { close, variants }, content)
}
