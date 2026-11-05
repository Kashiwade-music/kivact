//! Role: Date draft, validation and picker lifetime transitions.
use kivact_core::i18n::{CalendarDate, Locale};
use kivact_core::state::{Machine, Step};
#[derive(Clone, Copy, PartialEq)]
pub(super) enum OpenedBy { Input, Button }
#[derive(Clone, PartialEq)]
pub(super) enum Draft { Empty, Valid(CalendarDate), Invalid(String) }
#[derive(Clone, Copy, PartialEq)]
pub(super) enum DateValidation { Unchecked, Checked }
#[derive(Clone, PartialEq)]
pub(super) enum TypedDate { Following(CalendarDate), Editing { text: String, validation: DateValidation } }
#[derive(Clone, PartialEq)]
pub(super) enum DateInputState {
    Closed { draft: Draft },
    Open { opened_by: OpenedBy, draft: Draft },
}
impl DateInputState {
    pub(super) fn draft(&self) -> &Draft { match self { Self::Closed { draft } | Self::Open { draft, .. } => draft } }
    pub(super) fn is_open(&self) -> bool { matches!(self, Self::Open { .. }) }
}
pub(super) enum Event { Open(OpenedBy), Close, Commit(String), Pick(CalendarDate), Clear }
pub(super) struct Snapshot { pub(super) locale: Locale, pub(super) min: Option<CalendarDate>, pub(super) max: Option<CalendarDate> }
impl Machine for Snapshot {
    type State = DateInputState;
    type Event = Event;
    type Effect = Option<CalendarDate>;
    fn transition(&self, state: &DateInputState, event: Event) -> Step<DateInputState, Event, Option<CalendarDate>> {
        let accept = |date: CalendarDate| self.min.is_none_or(|min| date >= min) && self.max.is_none_or(|max| date <= max);
        match event {
            Event::Open(opened_by) => Step::to(DateInputState::Open { opened_by, draft: state.draft().clone() }),
            Event::Close => Step::to(DateInputState::Closed { draft: state.draft().clone() }),
            Event::Clear => Step::to(DateInputState::Closed { draft: Draft::Empty }).with(None),
            Event::Pick(date) if accept(date) => Step::to(DateInputState::Closed { draft: Draft::Valid(date) }).with(Some(date)),
            Event::Pick(_) => Step::stay(),
            Event::Commit(text) if text.trim().is_empty() => Step::to(DateInputState::Closed { draft: Draft::Empty }).with(None),
            Event::Commit(text) => match self.locale.parse_date(&text).filter(|date| accept(*date)) {
                Some(date) => Step::to(DateInputState::Closed { draft: Draft::Valid(date) }).with(Some(date)),
                None => Step::to(match state { DateInputState::Closed { .. } => DateInputState::Closed { draft: Draft::Invalid(text) }, DateInputState::Open { opened_by, .. } => DateInputState::Open { opened_by: *opened_by, draft: Draft::Invalid(text) } }),
            },
        }
    }
}
