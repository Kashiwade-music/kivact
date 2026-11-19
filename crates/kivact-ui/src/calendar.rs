//! Role: Gregorian calendar with shared keyboard and range-selection behavior.

use kivact_core::i18n::CalendarDate;
pub use view::{Calendar, calendar};
pub(crate) mod look;

mod caption;
mod navigation;
mod panel;
pub(crate) mod state;
mod view;

/// How the title of each month is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CalendarCaption {
    /// A text such as `October 2026`.
    #[default]
    Label,
    /// A month select and a year select.
    Dropdown,
}

kivact_core::state::constant_value!(CalendarCaption);

/// One colored event mark; the application's scheduling model owns its contents.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub date: CalendarDate,
    pub color: kivact_core::style::Color,
}
