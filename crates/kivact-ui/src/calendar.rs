//! Role: Gregorian calendar with shared keyboard and range-selection behavior.

mod caption;
pub(crate) mod look;
mod model;
mod navigation;
mod panel;
pub(crate) mod state;
pub(crate) mod tokens;
mod view;

pub use model::{CalendarCaption, CalendarEvent};
pub use view::{Calendar, calendar};
