//! Role: Gregorian calendar with shared keyboard and range-selection behavior.
mod caption;
mod panel;
mod state;
mod view;
pub use state::CalendarCaption;
pub use view::{Calendar, calendar};
