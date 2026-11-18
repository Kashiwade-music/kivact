//! Role: Discover running targets and open a DevTools connection (feature client).

mod connection;
mod discovery;
mod error;

pub use connection::*;
pub use discovery::*;
pub use error::*;
