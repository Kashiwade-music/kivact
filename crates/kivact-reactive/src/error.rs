//! Role: `UiError`, handler results and misuse panics.
//!
//! Handlers and callbacks return `()` or `Result<(), UiError>` (sealed in
//! [`HandlerResult`]; key handlers in `kivact-core` may also return
//! `Handled`), so `?` infers [`UiError`] without an annotation.
//!
//! A misuse (a setup-only call outside setup, a disposed handle, a write in
//! a pure computation, ...) is thrown as a `MisusePanic(UiError)` located at
//! the caller. The boundary that catches it (setup, handler, effect, task)
//! walks up the owner tree to the nearest `error_boundary` (its `on_error`,
//! then its fallback) or, without one, to the App's default sink
//! (`tracing::error!` and the diagnostics ring; see `report`). The panic
//! hook stays silent for these payloads.

use crate::AppId;
use std::any::Any;
use std::fmt;
use std::panic::Location;
use std::sync::{Arc, Once};

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
    /// A panic caught at a boundary (setup, handler, effect, task).
    Panic,
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

    /// The error for a panic `payload` caught at a boundary: a misuse panic
    /// carries its own located error; any other panic becomes
    /// [`ErrorKind::Panic`] located at `at`.
    pub fn from_panic(
        payload: Box<dyn Any + Send>,
        at: Option<&'static Location<'static>>,
    ) -> Self {
        match payload.downcast::<MisusePanic>() {
            Ok(m) => m.0,
            Err(payload) => {
                let text = if let Some(s) = payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "panic".to_string()
                };
                Self::new(ErrorKind::Panic, format!("panicked: {text}"), at)
            }
        }
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

/// The error for a caught panic `payload`; a misuse panic carries its own
/// located error, any other panic becomes `ErrorKind::Panic` at `at`.
pub(crate) fn error_from_panic(
    payload: Box<dyn Any + Send>,
    at: Option<&'static Location<'static>>,
) -> UiError {
    UiError::from_panic(payload, at)
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

/// Panic payload for detected misuse. Caught at boundaries and reported.
pub(crate) struct MisusePanic(pub(crate) UiError);

/// Raise a located error of `kind`: a misuse diagnostic of a component or
/// a layer. Inside an App the nearest boundary (setup, handler, effect,
/// task) catches it and reports it, and the panic hook stays quiet; outside
/// every App it is a panic with a readable message.
pub fn raise(kind: ErrorKind, message: impl Into<String>, at: &'static Location<'static>) -> ! {
    misuse_at(kind, message, at)
}

pub(crate) fn misuse_at(
    kind: ErrorKind,
    message: impl Into<String>,
    at: &'static Location<'static>,
) -> ! {
    resume(UiError::new(kind, message, Some(at)))
}

/// Raise `err`. Inside an App (a boundary will catch it) the payload is the
/// structured error and the panic hook stays quiet; outside every App no
/// boundary exists, so the payload is a readable `String` that the default
/// hook prints.
pub(crate) fn resume(err: UiError) -> ! {
    install_quiet_hook();
    if crate::execution::executing_app() == AppId::NONE {
        std::panic::panic_any(format!("kivact misuse: {err:?}"))
    }
    std::panic::panic_any(MisusePanic(err))
}

/// Misuse panics are reported through the error sink, not the panic hook.
/// Other panics keep the previously installed hook's behaviour.
pub(crate) fn install_quiet_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if info.payload().downcast_ref::<MisusePanic>().is_none() {
                previous(info);
            }
        }));
    });
}

pub(crate) mod sealed {
    //! Role: Sealed supertraits that keep `HandlerResult` implementable only here.
    /// Implemented for `()` and `Result<(), UiError>` only.
    pub trait Sealed {}
}

/// Return type of handlers and callbacks: `()` or `Result<(), UiError>`.
/// Handlers and callbacks run untracked: a `get` in one subscribes nothing.
/// Sealed with exactly these two impls so that `?` infers `UiError`.
#[diagnostic::on_unimplemented(
    message = "a handler must return `()` or `Result<(), UiError>`",
    note = "use `?` on other errors; they convert to `UiError`"
)]
pub trait HandlerResult: sealed::Sealed {
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

impl sealed::Sealed for () {}
impl sealed::Sealed for Result<(), UiError> {}
