//! Role: Decide range choices and external binding changes in one transition.
use kivact_core::i18n::{CalendarDate, DateRange};
use kivact_core::state::{Machine as MachineTrait, Step};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum CalendarRangePhase {
    Start,
    End {
        anchor: CalendarDate,
        before: Option<DateRange>,
    },
}
impl CalendarRangePhase {
    pub(super) fn current(self, value: Option<DateRange>) -> Self {
        match self {
            Self::End { anchor, .. } if value != Some(DateRange::new(anchor, anchor)) => {
                Self::Start
            }
            phase => phase,
        }
    }
}
pub(super) enum Event {
    ExternalValueChanged(Option<DateRange>),
    Choose {
        day: CalendarDate,
        current: Option<DateRange>,
    },
    Cancel(Option<DateRange>),
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = CalendarRangePhase;
    type Event = Event;
    type Effect = Option<DateRange>;
    fn transition(
        &self,
        state: &Self::State,
        event: Event,
    ) -> Step<Self::State, Event, Self::Effect> {
        match event {
            Event::ExternalValueChanged(current) => Step::to(state.current(current)),
            Event::Choose { day, current } => match state.current(current) {
                CalendarRangePhase::Start => Step::to(CalendarRangePhase::End {
                    anchor: day,
                    before: current,
                })
                .with(Some(DateRange::new(day, day))),
                CalendarRangePhase::End { anchor, .. } => {
                    Step::to(CalendarRangePhase::Start).with(Some(DateRange::new(anchor, day)))
                }
            },
            Event::Cancel(current) => match state.current(current) {
                CalendarRangePhase::Start => Step::to(CalendarRangePhase::Start),
                CalendarRangePhase::End { before, .. } => {
                    Step::to(CalendarRangePhase::Start).with(before)
                }
            },
        }
    }
}
