//! Role: The DevTools target: answers the agent's calls from the App's internals.

mod a11y;
pub(crate) mod apply;
mod console;
mod convert;
mod emulation;
mod hooks;
mod input;
pub(crate) mod inspect_mode;
mod lifecycle;
mod motion;
mod node;
mod overlay;
mod perf;
mod query;
mod review;
pub(crate) mod start;
pub(crate) mod state;
mod styles;
mod target;
mod tree;
mod turn;
mod updates;

pub(crate) use turn::{after_turn, run_events, run_incoming, take_incoming};
