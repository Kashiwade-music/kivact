//! Role: Decide range choices and external binding changes in one transition.
use kivact_core::i18n::{CalendarDate, DateRange};
use kivact_core::state::{Machine as MachineTrait, Step};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum RangePick {
    Idle,
    Picking {
        anchor: CalendarDate,
        preview: Option<CalendarDate>,
        before: Option<DateRange>,
    },
}
impl RangePick {
    pub(super) fn current(self, value: Option<DateRange>) -> Self {
        match self {
            Self::Picking { anchor, .. } if value != Some(DateRange::new(anchor, anchor)) => {
                Self::Idle
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
    Preview(Option<CalendarDate>),
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = RangePick;
    type Event = Event;
    type Effect = Option<DateRange>;
    fn transition(
        &self,
        state: &Self::State,
        event: Event,
    ) -> Step<Self::State, Event, Self::Effect> {
        match event {
            Event::Preview(preview) => match *state {
                RangePick::Idle => Step::stay(),
                RangePick::Picking { anchor, before, .. } => Step::to(RangePick::Picking {
                    anchor,
                    before,
                    preview,
                }),
            },
            Event::ExternalValueChanged(current) => Step::to(state.current(current)),
            Event::Choose { day, current } => match state.current(current) {
                RangePick::Idle => Step::to(RangePick::Picking {
                    anchor: day,
                    preview: None,
                    before: current,
                })
                .with(Some(DateRange::new(day, day))),
                RangePick::Picking { anchor, .. } => {
                    Step::to(RangePick::Idle).with(Some(DateRange::new(anchor, day)))
                }
            },
            Event::Cancel(current) => match state.current(current) {
                RangePick::Idle => Step::to(RangePick::Idle),
                RangePick::Picking { before, .. } => Step::to(RangePick::Idle).with(before),
            },
        }
    }
}
