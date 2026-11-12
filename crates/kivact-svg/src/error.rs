//! Role: The error of an SVG export, by the kind the host reports it as.

use std::fmt;

/// Why a drawing cannot be written as SVG. Each variant carries a message
/// that names the command, value or resource at fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SvgError {
    /// A command, value or property that SVG 1.1 cannot express.
    Unsupported(String),
    /// The drawing does not nest (a pop without its push, or a push left
    /// open), or an image it draws is too large to embed.
    Malformed(String),
    /// An image or a text failed to load, decode, rasterize or measure.
    Resource(String),
}

impl SvgError {
    /// The error of a command or value that SVG 1.1 cannot express.
    pub(crate) fn unsupported(what: &str) -> Self {
        Self::Unsupported(format!("SVG cannot express {what}"))
    }
}

impl fmt::Display for SvgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Unsupported(message) | Self::Malformed(message) | Self::Resource(message) => {
                message
            }
        };
        f.write_str(message)
    }
}

impl std::error::Error for SvgError {}
