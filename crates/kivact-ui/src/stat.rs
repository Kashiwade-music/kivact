//! Role: Present a named statistic with optional delta, icon and chart slots.
pub(crate) mod look;
pub use look::Trend;
mod view;
pub use view::{Stat, stat};
