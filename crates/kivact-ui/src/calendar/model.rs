//! Role: The calendar's public caption and colored event values.

use kivact_core::i18n::CalendarDate;
use kivact_macros::Prop;

/// How the title of each month is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum CalendarCaption {
    /// A text such as `October 2026`.
    #[default]
    Label,
    /// A month select and a year select.
    Dropdown,
}

/// One colored event mark; the application's scheduling model owns its contents.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub date: CalendarDate,
    pub color: kivact_core::style::Color,
}
