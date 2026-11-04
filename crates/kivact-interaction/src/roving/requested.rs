//! Role: One replaceable binding owns requested roving focus.
use kivact_core::custom::require_setup;
use kivact_core::state::{Binding, IntoBinding, Signal, signal};

pub(crate) struct Requested<K: kivact_core::state::ItemKey>(Signal<Binding<Option<K>>>);
impl<K: kivact_core::state::ItemKey> Clone for Requested<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for Requested<K> {}
impl<K: kivact_core::state::ItemKey> Requested<K> {
    pub(super) fn new() -> Self {
        Self(signal(signal(None).into_binding()))
    }
    pub(crate) fn bind(self, binding: Binding<Option<K>>) {
        require_setup("Roving::shared_highlight");
        self.0.update(|slot| *slot = binding);
    }
    pub(crate) fn get(self) -> Option<K> {
        self.0.get().get()
    }
    pub(crate) fn get_untracked(self) -> Option<K> {
        self.0.get_untracked().get_untracked()
    }
    pub(crate) fn set(self, key: Option<K>) {
        self.0.get_untracked().set(key);
    }
}
