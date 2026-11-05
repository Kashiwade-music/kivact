//! Role: State shared by the calendar's month panels.

use kivact_core::i18n::{CalendarDate, CalendarMonth};
use kivact_interaction::CalendarGridApi;
use kivact_core::state::{Binding, Value};

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
    pub(super) today: Value<CalendarDate>,
    pub(super) variable_rows: Value<bool>,
    pub(super) marks: std::rc::Rc<dyn Fn(CalendarDate) -> Option<kivact_interaction::Tone>>,
}

kivact_core::state::constant_value!(CalendarCaption);
