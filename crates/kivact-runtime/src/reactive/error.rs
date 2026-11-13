//! Role: `UiError`, handler results and misuse panics.
//!
//! Handlers and callbacks return `()` or `Result<(), UiError>` (sealed in
//! [`crate::HandlerResult`]; event handlers may also return
//! `Handled`), so `?` infers [`UiError`] without an annotation.
//!
//! A misuse (a setup-only call outside setup, a disposed handle, a write in
//! a pure computation, ...) is a bug in the caller, not an error value: it
//! panics at the caller with a located message, `kivact misuse: UiError(..)`,
//! and nothing catches it. The panic ends the App (or fails the test) with
//! Rust's usual message. Only `Err` values travel the error path: they go to
//! the nearest `error_boundary`'s `on_error`, or without one to the App's
//! default sink (`tracing::error!` and the diagnostics ring; see `report`).

use std::fmt;
use std::panic::Location;
use std::sync::Arc;

/// What went wrong. A misuse kind carries the data that locates the misuse
/// (such as where a disposed handle was created and disposed); the source
/// position is available through [`UiError::location`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A setup-only API was called outside component setup.
    SetupOnly {
        /// The API that was called.
        api: &'static str,
    },
    /// A handle was used after its owner was disposed.
    Disposed {
        // xtask-allow(state-fields): option: absent means the location is unknown
        /// Where the handle's node, object or owner was created, if known.
        // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
        created_at: Option<&'static Location<'static>>,
        // xtask-allow(state-fields): option: absent means the location is unknown
        /// Where the owner that released it was created, if known.
        // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
        disposed_by: Option<&'static Location<'static>>,
    },
    /// A write to a disposed signal (ignored, recorded in diagnostics).
    WriteAfterDispose,
    /// A signal write inside a memo or render effect.
    WriteInPureContext,
    /// A write (or read) while the same signal is borrowed by `with`/`update`.
    BorrowedWrite,
    /// A dependency or effect cycle; `path` lists creation sites.
    Cycle {
        /// Creation sites of the computations on the cycle.
        // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
        path: Vec<&'static Location<'static>>,
    },
    /// Duplicate `for_each` key; later rows are dropped. `key_hash` identifies the key.
    DuplicateKey {
        /// Hash of the duplicate key.
        key_hash: u64,
    },
    /// A handle from another App (or an App that no longer exists).
    WrongApp,
    /// An error made from a message ([`UiError::msg`]).
    Message,
    /// Converted from a `std::error::Error`.
    External,
    /// A deadline passed.
    Timeout,
    /// A request or a run was cancelled.
    Cancelled,
    /// The host cannot do this (capability table).
    Unsupported,
    /// A platform request (clipboard, focus, dialog) failed.
    Platform,
    /// The embedded App panicked (`kivact::embed`); the message is the
    /// panic's.
    Panicked,
    /// An action is closed, or its queued runs were abandoned at the drain
    /// deadline.
    Closed,
    /// An image or a font could not be used: a file that is missing or
    /// unreadable, an unsupported or broken format, a limit exceeded, a
    /// font family that is not installed. The message names the source
    /// (for a file, its absolute path and the current directory) and the
    /// reason; the location is where the node or the request was created
    /// (`image(..)`, `App::font`).
    ///
    /// It is a diagnostic, not a failure of the subtree: it goes to the
    /// App's default sink (`tracing` at target `kivact`, and the
    /// diagnostics ring that `kivact::test`'s `Screen::errors` reads),
    /// never to an `error_boundary`'s `on_error` or fallback. The node
    /// draws nothing (an image) or uses the system UI font (a family), and
    /// the rest of the App goes on. An image is reported once per node and
    /// image, a family once per family. Images drawn on a canvas with
    /// `Draw::image` are never reported.
    Asset,
    /// A shader that did not validate, or a shader parameter that does not
    /// match the shader's `Params`. The message starts with
    /// `"line L, column C: "` when it has a position in the source.
    Shader,
}

struct Inner {
    kind: ErrorKind,
    message: String,
    // xtask-allow(state-fields): option: absent means the error has no source position
    at: Option<&'static Location<'static>>,
    // xtask-allow(state-fields): option: absent means the error has no cause
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// The one error type of Kivact. Cheap to clone (`Arc`), `Display`, and
/// convertible from any `std::error::Error + Send + Sync`, so `?` works in
/// handlers without annotations. It does not implement `std::error::Error`
/// itself, because the blanket `From` would then overlap `From<T> for T`; use
/// [`UiError::as_error`] or convert into `Box<dyn Error + Send + Sync>` with
/// `?`. `PartialEq` is identity: a clone equals its original, two errors with
/// the same kind and message do not.
#[derive(Clone)]
pub struct UiError(Arc<Inner>);

impl UiError {
    /// An error of `kind` with `message`, located at `at` when known.
    // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
    pub fn new(
        kind: ErrorKind,
        message: impl Into<String>,
        at: Option<&'static Location<'static>>,
    ) -> Self {
        UiError(Arc::new(Inner {
            kind,
            message: message.into(),
            at,
            source: None,
        }))
    }

    /// An error with a message.
    #[track_caller]
    pub fn msg(message: impl fmt::Display) -> Self {
        Self::new(
            ErrorKind::Message,
            message.to_string(),
            Some(Location::caller()),
        )
    }

    /// Convert a boxed error (for example `anyhow::Error::into()`).
    /// A [`UiError`] boxed with `Box::from` converts back to the same error.
    #[track_caller]
    // xtask-allow(api-shape): Standard Rust Error interop intentionally accepts boxed errors.
    pub fn from_boxed(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        let err = match err.downcast::<Boxed>() {
            Ok(boxed) => return boxed.0,
            Err(err) => err,
        };
        UiError(Arc::new(Inner {
            kind: ErrorKind::External,
            message: err.to_string(),
            at: Some(Location::caller()),
            source: Some(err),
        }))
    }

    /// This error as a standard error: `Display` is the message and
    /// `source()` the wrapped error when it was converted from one. Use it
    /// with APIs that take `&dyn Error` (logging, error reporters).
    // xtask-allow(api-shape): Standard Rust Error interop intentionally exposes the std trait object.
    pub fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        &*self.0
    }

    /// What went wrong.
    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }

    /// Whether this error reports a cancellation (`ErrorKind::Cancelled`):
    /// a resource request or an action run that was cancelled ends in
    /// `AsyncState::Failed` with such an error.
    pub fn is_cancelled(&self) -> bool {
        self.0.kind == ErrorKind::Cancelled
    }

    /// The readable message.
    pub fn message(&self) -> &str {
        &self.0.message
    }

    /// Source position where the error was raised, when known.
    // xtask-allow(api-shape): Diagnostic source positions retain Rust caller locations.
    pub fn location(&self) -> Option<&'static Location<'static>> {
        self.0.at
    }

    /// The wrapped `std::error::Error`, if converted from one.
    pub fn source_error(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.0.source.as_deref()
    }
}

/// A located error of `kind` (the crate-internal constructor of [`UiError`]).
pub(crate) fn new_error(
    kind: ErrorKind,
    message: impl Into<String>,
    at: Option<&'static Location<'static>>,
) -> UiError {
    UiError::new(kind, message, at)
}

/// The `std::error::Error` view of a [`UiError`] (see [`UiError::as_error`]).
impl std::error::Error for Inner {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl fmt::Display for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UiError({:?}: {}", self.kind, self.message)?;
        if let Some(at) = self.at {
            write!(f, " at {}:{}", at.file(), at.line())?;
        }
        f.write_str(")")
    }
}

/// A [`UiError`] boxed as a standard error; `UiError::from_boxed` unwraps it.
struct Boxed(UiError);

impl fmt::Display for Boxed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for Boxed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Boxed {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.as_error().source()
    }
}

/// `?` into `Box<dyn Error + Send + Sync>` (and into `anyhow`/`eyre` through
/// it). Keeps the kind, message and location; `UiError::from_boxed` returns
/// the original error.
impl From<UiError> for Box<dyn std::error::Error + Send + Sync + 'static> {
    fn from(error: UiError) -> Self {
        Box::new(Boxed(error))
    }
}

/// `?` into `Box<dyn Error>`.
impl From<UiError> for Box<dyn std::error::Error + 'static> {
    fn from(error: UiError) -> Self {
        Box::new(Boxed(error))
    }
}

impl AsRef<dyn std::error::Error + Send + Sync + 'static> for UiError {
    fn as_ref(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self.as_error()
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<E> for UiError {
    #[track_caller]
    fn from(err: E) -> Self {
        UiError(Arc::new(Inner {
            kind: ErrorKind::External,
            message: err.to_string(),
            at: Some(Location::caller()),
            source: Some(Box::new(err)),
        }))
    }
}

/// Two errors are equal when they are the same error instance (clones).
impl PartialEq for UiError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// `std::result::Result` with [`UiError`].
pub type Result<T, E = UiError> = std::result::Result<T, E>;

/// Raise a located error of `kind`: a misuse diagnostic of a component or
/// a layer. It always panics at the caller with a readable message
/// (`kivact misuse: UiError(..)`); nothing catches it, so it ends the App
/// (or fails the test).
pub fn raise(kind: ErrorKind, message: impl Into<String>, at: &'static Location<'static>) -> ! {
    misuse_at(kind, message, at)
}

pub(crate) fn misuse_at(
    kind: ErrorKind,
    message: impl Into<String>,
    at: &'static Location<'static>,
) -> ! {
    misuse(UiError::new(kind, message, Some(at)))
}

/// Panic with `err`: a detected misuse is a bug, never an error value.
pub(crate) fn misuse(err: UiError) -> ! {
    panic!("kivact misuse: {err:?}")
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the standard-error interop of `UiError`.
    use super::{ErrorKind, UiError};
    use std::error::Error;

    #[test]
    fn a_ui_error_is_a_standard_error_with_its_message_and_source() {
        let error = UiError::msg("x");
        assert_eq!(error.as_error().to_string(), "x");
        assert!(error.as_error().source().is_none());

        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let converted = UiError::from(io);
        assert_eq!(converted.kind(), &ErrorKind::External);
        let source = converted
            .as_error()
            .source()
            .expect("a converted error keeps its source");
        assert_eq!(source.to_string(), "missing");
    }

    #[test]
    fn question_mark_boxes_a_ui_error_and_from_boxed_keeps_it() {
        fn fails() -> Result<(), Box<dyn Error + Send + Sync>> {
            Err(UiError::msg("x"))?;
            Ok(())
        }
        let boxed = fails().expect_err("the boxed error");
        assert_eq!(boxed.to_string(), "x");

        let original = UiError::msg("round trip");
        let boxed: Box<dyn Error + Send + Sync> = Box::from(original.clone());
        let back = UiError::from_boxed(boxed);
        assert_eq!(back, original);
        assert_eq!(back.kind(), original.kind());
        assert_eq!(back.location(), original.location());
    }
}
