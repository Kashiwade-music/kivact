//! Role: Wall-clock text editing and adaptive scalar wheels.
pub(crate) mod look;
pub(crate) mod parse;

mod view;
pub use view::{TimeInput, time_input};
