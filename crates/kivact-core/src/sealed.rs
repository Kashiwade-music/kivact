//! Role: Public-in-bounds traits that only this crate can name or implement.

/// Seal of `KeyHandlerResult`: implemented exactly for `()`, `Handled` and
/// `Result<Handled, UiError>`.
pub trait KeyHandlerResultSeal {}
