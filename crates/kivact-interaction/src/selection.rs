//! Role: Single and multiple selections share one binding input contract.

use kivact_core::state::{Binding, IntoBinding, Signal};

/// Selection cardinality is determined by the binding's value type.
pub enum Selection<K: 'static> {
    Single(Binding<Option<K>>),
    Multiple(Binding<Vec<K>>),
}
/// Pass a Signal or Binding of `Option<K>` for one key, or `Vec<K>` for many keys.
#[diagnostic::on_unimplemented(
    message = "selection needs a binding of Option<K> or Vec<K>",
    note = "pass a Signal or Binding of Option<K> (single) or Vec<K> (multiple)"
)]
pub trait IntoSelection<K: 'static>: crate::sealed::IntoSelectionSeal<K> {
    fn into_selection(self) -> Selection<K>;
}
impl<K: 'static> crate::sealed::IntoSelectionSeal<K> for Signal<Option<K>> {}
impl<K: 'static> crate::sealed::IntoSelectionSeal<K> for Signal<Vec<K>> {}
impl<K: 'static> crate::sealed::IntoSelectionSeal<K> for Binding<Option<K>> {}
impl<K: 'static> crate::sealed::IntoSelectionSeal<K> for Binding<Vec<K>> {}
impl<K: 'static> IntoSelection<K> for Signal<Option<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Single(self.into_binding())
    }
}
impl<K: 'static> IntoSelection<K> for Signal<Vec<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Multiple(self.into_binding())
    }
}
impl<K: 'static> IntoSelection<K> for Binding<Option<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Single(self)
    }
}
impl<K: 'static> IntoSelection<K> for Binding<Vec<K>> {
    fn into_selection(self) -> Selection<K> {
        Selection::Multiple(self)
    }
}
