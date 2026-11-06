//! Role: State shared by the calendar's month panels.

use crate::calendar_values::{CalendarCaption, CalendarEvent};
use kivact_core::i18n::{CalendarDate, CalendarMonth};
use kivact_core::state::{Binding, Value};
use kivact_interaction::CalendarGridApi;

/// Everything the month panels and their day cells share.
// xtask-allow(state-fields): Immutable setup configuration, independent slots and reactive presentation values.
pub(super) struct Shared {
    pub(super) swipe: kivact_core::state::MachineHandle<super::navigation::SwipeSnapshot>,
    pub(super) cell_size: f64,
    pub(super) variants: crate::variants::Variants,
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
