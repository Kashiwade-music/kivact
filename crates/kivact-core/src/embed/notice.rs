//! Role: Host notices of an embedded App.
//!
//! A notice is a wake-up, not a state: read
//! [`Embedded::status`](super::Embedded::status) or
//! [`Embedded::scale`](super::Embedded::scale) after one. See
//! [`Parent::on_notice`](super::Parent::on_notice) for the callback contract.

pub use kivact_engine::embed::EmbedNotice;
