//! Role: Month calendar with single and range selection and keyboard navigation.

mod caption;
mod panel;
mod state;
mod view;

pub(crate) use caption::Names;
pub use state::CalendarCaption;
pub use view::{Calendar, calendar, calendar_range};
