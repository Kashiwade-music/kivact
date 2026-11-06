//! Role: Public-in-bounds traits that only this crate can name or implement.

/// Implemented for `()` and `Result<(), UiError>` only.
pub trait HandlerResultSeal {}

/// Seal of `IntoText`: implemented exactly for the text inputs it lists.
pub trait IntoTextSeal {}

/// Writable input conversions supported by this runtime.
pub trait IntoBindingSeal {}
