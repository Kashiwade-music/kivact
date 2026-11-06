//! Role: Host event dispatch, tracing, and per-application boundary state.

pub(crate) mod dispatch;
pub(crate) mod notice;
pub(crate) mod state;
#[cfg(debug_assertions)]
pub(crate) mod trace;
