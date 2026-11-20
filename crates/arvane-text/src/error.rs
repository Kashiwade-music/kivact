//! Role: Defines text shaping errors.

use std::fmt;

/// An input or font availability error while shaping text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TextError {
    /// Font size, line height, or available width is invalid.
    InvalidMetrics,
    /// Supplied font data contains no usable font faces.
    InvalidFont,
    /// No usable system or registered font could shape the input.
    MissingFont,
    /// Rich text spans overlap, are out of order, or cut a character.
    InvalidSpans,
    /// An inline object cuts a character, has invalid size, or repeats an identity.
    InvalidInlineBoxes,
    /// The inline paragraph asks for ellipsis or a line clamp.
    UnsupportedInlineStyle,
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidMetrics => "invalid text metrics",
            Self::InvalidFont => "font data contains no usable faces",
            Self::MissingFont => "no usable font available for text",
            Self::InvalidInlineBoxes => "invalid inline object metrics, identity, or byte offset",
            Self::UnsupportedInlineStyle => "inline paragraphs cannot be ellipsized or clamped",
            Self::InvalidSpans => "text spans overlap, are out of order or cut a character",
        })
    }
}

impl std::error::Error for TextError {}

/// The shared failure of a text error: invalid input is `Content`, a
/// font that cannot be used is `Resource`.
pub(crate) fn failure(error: TextError) -> kivact_types::Failure {
    let kind = match error {
        TextError::InvalidMetrics
        | TextError::InvalidSpans
        | TextError::InvalidInlineBoxes
        | TextError::UnsupportedInlineStyle => kivact_types::FailureKind::Content,
        TextError::InvalidFont | TextError::MissingFont => kivact_types::FailureKind::Resource,
    };
    kivact_types::Failure::new(kind, error.to_string())
}
