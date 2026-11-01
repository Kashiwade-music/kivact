//! Role: Facade errors classified by how a host can recover.

use std::fmt;

/// How a host can recover from an [`Error`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Input the application supplied is invalid: scene geometry, paint or
    /// clips, frame size, or text metrics. Retrying the same input fails
    /// again.
    Content,
    /// An external resource cannot be used: font data, or no usable font
    /// for the text.
    Resource,
    /// The adapter or device failed: GPU adapter, device or readback.
    Backend,
    /// A window's presentation surface failed; recreate it.
    Surface,
    /// The capability is not available here: no hardware GPU adapter, or
    /// GPU support not built in.
    Unavailable,
}

/// Adapter failure with a stable, external-library-independent message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    /// Describe an environmental failure at an application or adapter boundary.
    ///
    /// The error is classified as [`ErrorKind::Backend`].
    pub fn new(message: impl Into<String>) -> Self {
        Self::with(ErrorKind::Backend, message)
    }

    /// How a host can recover from this failure.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub(crate) fn with(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}

pub(crate) fn text_kind(error: &arvane_text::TextError) -> ErrorKind {
    match error {
        arvane_text::TextError::InvalidMetrics | arvane_text::TextError::InvalidSpans => {
            ErrorKind::Content
        }
        arvane_text::TextError::InvalidFont | arvane_text::TextError::MissingFont => {
            ErrorKind::Resource
        }
    }
}

pub(crate) fn text(error: arvane_text::TextError) -> Error {
    Error::with(text_kind(&error), error.to_string())
}

/// Rendering failures; an invalid image in a scene is invalid content.
pub(crate) fn render(error: arvane_render::RenderError) -> Error {
    let kind = match &error {
        arvane_render::RenderError::Backend(_) => ErrorKind::Backend,
        arvane_render::RenderError::NoAdapter(_) => ErrorKind::Unavailable,
        arvane_render::RenderError::Surface(_) => ErrorKind::Surface,
        _ => ErrorKind::Content,
    };
    Error::with(kind, error.to_string())
}
