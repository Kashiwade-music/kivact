//! Role: The error a shared image's call returns when it does not do what it asked.

use std::fmt;

/// Why a call of a `ShareLink` did not do what it asked.
#[derive(Clone, Debug, PartialEq)]
pub enum ShareError {
    /// The request was replaced by a newer one: drop what was made for
    /// it and take the newer one (`ShareLink::poll_request`).
    Stale,
    /// The shared image's owner is gone: stop.
    Closed,
    /// The wait ran out of time.
    Timeout,
    /// The request cannot be served so: the window refused the textures
    /// (its message), a `ShareMode::Pixels` request was given textures,
    /// or a request was given textures twice.
    Failed(String),
}

impl fmt::Display for ShareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShareError::Stale => f.write_str("the share request was replaced by a newer one"),
            ShareError::Closed => f.write_str("the shared image is closed"),
            ShareError::Timeout => f.write_str("the wait for the shared image timed out"),
            ShareError::Failed(message) => write!(f, "the shared image failed: {message}"),
        }
    }
}

impl std::error::Error for ShareError {}
