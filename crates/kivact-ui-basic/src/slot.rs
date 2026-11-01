//! Role: The deferred view a builder's slot method keeps until mount.

use kivact::view::{IntoView, View};

/// A view a builder's slot method builds once, when the component mounts:
/// the closure runs when the component's own view takes the slot.
pub(crate) struct Slot(Box<dyn FnOnce() -> View>);

impl IntoView for Slot {
    fn into_view(self) -> View {
        (self.0)()
    }
}

/// The [`Slot`] of a slot method's closure.
pub(crate) fn slot<V: IntoView>(build: impl FnOnce() -> V + 'static) -> Slot {
    Slot(Box::new(move || build().into_view()))
}
