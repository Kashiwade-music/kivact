//! Role: Public-in-bounds traits that only this crate can name or implement.

/// Implemented for `()` and `Result<(), UiError>` only.
pub trait HandlerResultSeal {}

/// Seal of `IntoHandler`: nullary closures and `Callback<()>` only (DEC-16).
pub trait IntoHandlerSeal<E, M> {}

/// Seal of `IntoText`: implemented exactly for the text inputs it lists.
pub trait IntoTextSeal {}
