//! Role: Present a named statistic with optional delta, icon and chart slots.

pub(crate) mod look;
mod view;

pub use look::Trend;
pub use view::{Stat, stat};
