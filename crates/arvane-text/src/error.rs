//! Role: Defines text shaping errors.

use std::fmt;

/// An input or font availability error while shaping text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextError {
    /// Font size, line height, or available width is invalid.
    InvalidMetrics,
    /// Supplied font data contains no usable font faces.
    InvalidFont,
    /// No usable system or registered font could shape the input.
    MissingFont,
    /// Rich text spans overlap, are out of order, or cut a character.
    InvalidSpans,
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidMetrics => "invalid text metrics",
            Self::InvalidFont => "font data contains no usable faces",
            Self::MissingFont => "no usable font available for text",
            Self::InvalidSpans => "text spans overlap, are out of order or cut a character",
        })
    }
}

impl std::error::Error for TextError {}
