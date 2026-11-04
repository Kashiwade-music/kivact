//! Role: State shared by the calendar's month panels.

use kivact::i18n::{CalendarDate, CalendarMonth};
use kivact::interaction::CalendarGridApi;
use kivact::state::{Binding, Value};

/// How the title of each month is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CalendarCaption {
    /// A text such as `October 2026`.
    #[default]
    Label,
    /// A month select and a year select.
    Dropdown,
}

/// Everything the month panels and their day cells share.
pub(super) struct Shared {
    /// The keyboard, selection and paging behavior.
    pub(super) grid: CalendarGridApi,
    /// The first shown month.
    pub(super) month: Binding<CalendarMonth>,
    pub(super) count: usize,
    pub(super) caption: CalendarCaption,
    pub(super) outside: Value<bool>,
    pub(super) today: CalendarDate,
}

kivact::state::constant_value!(CalendarCaption);
