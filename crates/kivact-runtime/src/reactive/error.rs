//! Role: `UiError`, handler results and misuse panics.
//!
//! Handlers and callbacks return `()` or `Result<(), UiError>` (sealed in
//! [`HandlerResult`]; key handlers in `kivact-core` may also return
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

/// What went wrong. Misuse kinds carry the data listed in the design's misuse
/// table; the source position is available through [`UiError::location`].
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
        /// Where the handle's node, object or owner was created, if known.
        created_at: Option<&'static Location<'static>>,
        /// Where the owner that released it was created, if known.
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
}

struct Inner {
    kind: ErrorKind,
    message: String,
    at: Option<&'static Location<'static>>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

/// The one error type of Kivact. Cheap to clone (`Arc`), `Display`, and
/// convertible from any `std::error::Error + Send + Sync`, so `?` works in
/// handlers without annotations. It deliberately does not implement
/// `std::error::Error` (that would conflict with the blanket `From`).
/// `PartialEq` is identity: a clone equals its original, two errors with
/// the same kind and message do not.
#[derive(Clone)]
pub struct UiError(Arc<Inner>);

impl UiError {
    /// An error of `kind` with `message`, located at `at` when known.
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
    #[track_caller]
    pub fn from_boxed(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        UiError(Arc::new(Inner {
            kind: ErrorKind::External,
            message: err.to_string(),
            at: Some(Location::caller()),
            source: Some(err),
        }))
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
        f.write_str(&self.0.message)
    }
}

impl fmt::Debug for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UiError({:?}: {}", self.0.kind, self.0.message)?;
        if let Some(at) = self.0.at {
            write!(f, " at {}:{}", at.file(), at.line())?;
        }
        f.write_str(")")
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

/// Return type of handlers and callbacks: `()` or `Result<(), UiError>`.
/// Handlers and callbacks run untracked: a `get` in one subscribes nothing.
/// Sealed with exactly these two impls so that `?` infers `UiError`.
#[diagnostic::on_unimplemented(
    message = "a handler must return `()` or `Result<(), UiError>`",
    note = "use `?` on other errors; they convert to `UiError`"
)]
pub trait HandlerResult: crate::reactive::sealed::HandlerResultSeal {
    /// The result as `Result<(), UiError>`.
    fn into_result(self) -> Result<()>;
}

impl HandlerResult for () {
    fn into_result(self) -> Result<()> {
        Ok(())
    }
}

impl HandlerResult for Result<()> {
    fn into_result(self) -> Result<()> {
        self
    }
}

impl crate::reactive::sealed::HandlerResultSeal for () {}
impl crate::reactive::sealed::HandlerResultSeal for Result<(), UiError> {}
