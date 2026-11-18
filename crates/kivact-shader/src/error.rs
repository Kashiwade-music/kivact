//! Role: The error of a user shader source, with its position in that source.

use std::fmt;

/// A shader source that cannot be assembled, and where it went wrong.
///
/// The position is 1-based in the user's source, with the column counted in
/// bytes. It is `None` when the error has no place in the source, such as a
/// missing entry point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderError {
    /// What is wrong, in English.
    pub message: String,
    /// The 1-based line and byte column in the user's source, or `None` when
    /// the error has no place in it.
    pub position: Option<(u32, u32)>,
}

impl ShaderError {
    /// Creates an error at a position of the user's source.
    pub(crate) fn new(message: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            message: message.into(),
            position: Some((line, column)),
        }
    }

    /// Creates an error that has no place in the user's source.
    pub(crate) fn whole(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: None,
        }
    }
}

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.position {
            None => write!(f, "{}", self.message),
            Some((line, column)) => write!(f, "{line}:{column}: {}", self.message),
        }
    }
}

impl std::error::Error for ShaderError {}
