//! Role: Reusable factories create a fresh view for each opening.

use super::description::{IntoView, View};
use std::rc::Rc;

/// Store a reusable view factory, converting a closure with `ViewFn::from`.
/// Equality compares factory identity: a clone is equal, a separately created factory is not.
pub struct ViewFn<A = ()>(Rc<dyn Fn(A) -> View>);
impl<A> PartialEq for ViewFn<A> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl<A> Clone for ViewFn<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<A, F, V> From<F> for ViewFn<A>
where
    F: Fn(A) -> V + 'static,
    V: IntoView,
{
    fn from(f: F) -> Self {
        Self(Rc::new(move |arg| f(arg).into_view()))
    }
}
impl<A> ViewFn<A> {
    pub fn call(&self, arg: A) -> View {
        (self.0)(arg)
    }
}

/// Content is created again each time it opens: pass `|| view`.
#[diagnostic::on_unimplemented(
    message = "this content must be created each time it opens",
    note = "pass a closure `|| view` returning a view"
)]
pub trait IntoViewFn<V>: super::sealed_view_fn::Sealed<V> {
    #[doc(hidden)]
    fn into_view_fn(self) -> ViewFn<()>;
}
impl<F: Fn() -> V + 'static, V: IntoView> IntoViewFn<V> for F {
    fn into_view_fn(self) -> ViewFn<()> {
        ViewFn::from(move |()| self())
    }
}
impl IntoViewFn<View> for ViewFn<()> {
    fn into_view_fn(self) -> ViewFn<()> {
        self
    }
}
impl super::sealed_view_fn::Sealed<View> for ViewFn<()> {}
