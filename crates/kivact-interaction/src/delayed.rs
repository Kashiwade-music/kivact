//! Role: Expose the delayed behavior and keep its transition rules private.
mod api;
mod machine;
pub use api::{DelayedApi, DelayedPhase, Delays, use_delayed};

pub(crate) use api::delayed;
