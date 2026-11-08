//! Role: Drawables in the model: mounted roots, the app's export asks, retries.
//!
//! A drawable root is a node whose properties carry a `DrawableRole`. The
//! model keeps one entry per such root (`state`), reconciled at each commit.
//! `export` holds the exports and GPU pools, `request` applies the app's
//! requests, `place` decides where a canvas draws each drawable and `hit`
//! tests them.

pub(crate) mod export;
pub(crate) mod hit;
pub(crate) mod place;
pub(crate) mod request;
pub(crate) mod state;

pub(crate) use export::export_of;
pub(crate) use request::apply_request;
