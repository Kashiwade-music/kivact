//! Role: The error of a DevTools client call.

use std::fmt;

use kivact_devtools_protocol::RpcError;

/// Why a call or a connection failed.
#[derive(Clone, Debug, PartialEq)]
pub enum ClientError {
    /// The socket failed, or a call took longer than 30 seconds (`Io("timed out")`).
    Io(String),
    /// The connection has ended.
    Closed,
    /// The agent answered with an error.
    Rpc(RpcError),
    /// The agent speaks something this client cannot read (including a different protocol version).
    Protocol(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Io(text) => write!(f, "I/O error: {text}"),
            ClientError::Closed => write!(f, "the connection was closed"),
            ClientError::Rpc(error) => write!(f, "{} ({})", error.message, error.code),
            ClientError::Protocol(text) => write!(f, "protocol error: {text}"),
        }
    }
}

impl std::error::Error for ClientError {}

pub(crate) fn io_error(error: std::io::Error) -> ClientError {
    ClientError::Io(error.to_string())
}
