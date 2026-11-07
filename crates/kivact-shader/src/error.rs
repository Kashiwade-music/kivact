//! Role: The error of a user shader source, with its position in that source.

use std::fmt;

/// A shader source that cannot be assembled, and where it went wrong.
///
/// `line` and `column` are 1-based positions in the user's source, with the
/// column counted in bytes. Both are 0 when the error has no place in the
/// source, such as a missing entry point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderError {
    /// What is wrong, in English.
    pub message: String,
    /// The 1-based line in the user's source, or 0 when there is none.
    pub line: u32,
    /// The 1-based byte column in that line, or 0 when there is none.
    pub column: u32,
}

impl ShaderError {
    /// Creates an error at a position of the user's source.
    pub(crate) fn new(message: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            message: message.into(),
            line,
            column,
        }
    }

    /// Creates an error that has no place in the user's source.
    pub(crate) fn whole(message: impl Into<String>) -> Self {
        Self::new(message, 0, 0)
    }
}

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}:{}: {}", self.line, self.column, self.message)
        }
    }
}

impl std::error::Error for ShaderError {}
