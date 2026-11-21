//! Role: Gregorian calendar with shared keyboard and range-selection behavior.

mod caption;
pub(crate) mod look;
mod model;
mod navigation;
mod panel;
pub(crate) mod state;
mod view;

pub use model::{CalendarCaption, CalendarEvent};
pub use view::{Calendar, calendar};
pub(crate) use view::{range_validator, sheet_month};
