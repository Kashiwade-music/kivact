//! Role: The native window host contract and its initial options.
//!
//! The engine's `run_native(client, options)` opens an Arvane window.
//! Client callbacks run without an outstanding model borrow; the caller
//! validates the initial size before starting the window loop.

pub use crate::host::options::HostOptions;
