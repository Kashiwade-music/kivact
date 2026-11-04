//! Role: Report component misuse at its declaration site.

/// Raise a component misuse at the caller.
#[track_caller]
pub fn raise(kind: crate::ErrorKind, message: impl Into<String>) -> ! {
    kivact_runtime::raise(kind, message, std::panic::Location::caller())
}
