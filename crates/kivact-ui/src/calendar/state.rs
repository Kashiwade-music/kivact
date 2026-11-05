//! Role: State shared by the calendar's month panels.

use kivact_core::i18n::{CalendarDate, CalendarMonth};
use kivact_core::state::{Binding, Value};
use kivact_interaction::CalendarGridApi;

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
// xtask-allow(state-fields): Immutable setup configuration, independent slots and reactive presentation values.
pub(super) struct Shared {
    /// The keyboard, selection and paging behavior.
    pub(super) grid: CalendarGridApi,
    /// The first shown month.
    pub(super) month: Binding<CalendarMonth>,
    pub(super) count: usize,
    pub(super) caption: CalendarCaption,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    pub(super) outside: Value<bool>,
    pub(super) today: Value<CalendarDate>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    pub(super) variable_rows: Value<bool>,
    pub(super) events: Value<Vec<CalendarEvent>>,
    pub(super) weekend:
        std::rc::Rc<dyn Fn(kivact_core::i18n::Weekday) -> Option<kivact_interaction::Tone>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    pub(super) year_navigation: Value<bool>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    pub(super) month_title: Option<std::rc::Rc<dyn Fn(CalendarDate) -> String>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    pub(super) range: Value<Option<kivact_core::i18n::DateRange>>,
    // xtask-allow(state-fields): Independent setup or presentation option, not a lifecycle flag.
    pub(super) marks: std::rc::Rc<dyn Fn(CalendarDate) -> Option<kivact_interaction::Tone>>,
}

kivact_core::state::constant_value!(CalendarCaption);

/// One colored event mark; the application's scheduling model owns its contents.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub date: CalendarDate,
    pub color: kivact_core::style::Color,
}

// xtask-allow(state-fields): Drawing projection of independent date membership, today and semantic mark facts.
/// The logical state supplied to a calendar day recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDayVisual {
    pub day: kivact_interaction::CalendarDayState,
    // xtask-allow(state-fields): Independent drawing fact for today.
    pub today: bool,
    // xtask-allow(state-fields): Independent drawing fact for month membership.
    pub in_month: bool,
    pub mark: Option<kivact_interaction::Tone>,
}
