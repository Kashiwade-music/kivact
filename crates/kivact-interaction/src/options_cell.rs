//! Role: Setup-only options with reactive values for mounted parts.
use kivact_core::custom::require_setup;
use kivact_core::state::{Signal, Value, signal};
pub(crate) struct OptionsCell<S: Clone + 'static>(Signal<S>);
impl<S: Clone + 'static> Clone for OptionsCell<S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<S: Clone + 'static> Copy for OptionsCell<S> {}
impl<S: Clone + 'static> OptionsCell<S> {
    pub(crate) fn new(initial: S) -> Self {
        Self(signal(initial))
    }
    #[track_caller]
    pub(crate) fn update(&self, f: impl FnOnce(&mut S)) {
        require_setup("OptionsCell::update");
        self.0.update(f);
    }
    pub(crate) fn get(&self) -> S {
        self.0.get_untracked()
    }
    pub(crate) fn value(&self) -> Value<S> {
        self.0.into()
    }
}
