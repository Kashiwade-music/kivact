//! Role: Style shared tooltip surface and trigger parts without owning timers.
pub(crate) mod look;
mod view;

pub use view::{Tooltip, tooltip};
