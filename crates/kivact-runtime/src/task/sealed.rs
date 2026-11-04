//! Role: Public-in-bounds traits that only this crate can name or implement.

/// Seal of `ActionFn`, with the form `M` of its run closure.
pub trait ActionFnSeal<M> {}

/// Seal of `BlockingWork`, with the form `M` of its closure.
pub trait BlockingWorkSeal<M> {}

/// The form of a closure without an argument.
pub struct Nullary;

/// The form of a closure with one argument of type `I`.
pub struct Unary<I>(std::marker::PhantomData<I>);
