//! Role: The DevTools target: answers the agent's calls from the App's internals.

mod a11y;
mod console;
mod convert;
mod emulation;
mod input;
mod motion;
mod node;
mod overlay;
mod perf;
mod query;
mod review;
mod scripts;
pub(crate) mod start;
mod styles;
mod target;
mod tree;
mod turn;
mod updates;

pub(crate) use turn::{after_turn, run_events, run_incoming, take_incoming};
