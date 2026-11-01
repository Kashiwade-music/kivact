//! Role: The deferred view a builder's slot method keeps until mount.

/// A view a builder's slot method builds once, when the component mounts.
pub(super) type Slot = Box<dyn FnOnce() -> kivact::view::View>;

/// The [`Slot`] of a slot method's closure.
pub(super) fn slot<V: kivact::view::IntoView>(build: impl FnOnce() -> V + 'static) -> Slot {
    Box::new(move || build().into_view())
}
