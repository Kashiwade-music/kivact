//! Role: Limit reusable view factories to closures.

pub trait Sealed<V> {}
impl<F: Fn() -> V + 'static, V: super::description::IntoView> Sealed<V> for F {}
