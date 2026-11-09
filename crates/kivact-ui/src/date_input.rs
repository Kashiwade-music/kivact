//! Role: Locale-sensitive date editing with a calendar or a touch sheet.
mod machine;
mod view;
pub(crate) use machine::{OpenedBy, Picker};
pub use view::{DateInput, DatePresentation, date_input};
