//! Role: Gregorian calendar with shared keyboard and range-selection behavior.
mod caption;
mod navigation;
mod panel;
pub(crate) mod state;
mod view;
pub use state::{CalendarCaption, CalendarDayVisual, CalendarEvent};
pub use view::{Calendar, calendar};
