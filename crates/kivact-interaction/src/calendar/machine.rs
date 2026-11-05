//! Role: Decide range choices and external binding changes in one transition.
use kivact_core::i18n::{CalendarDate, DateRange};
use kivact_core::state::{Machine as MachineTrait, Step};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum RangePick {
    Idle,
    Picking {
        anchor: CalendarDate,
        preview: Option<CalendarDate>,
    },
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
pub(super) enum Effect {
    Commit(DateRange),
    Started(CalendarDate),
    Cancelled,
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = RangePick;
    type Event = Event;
    type Effect = Effect;
    fn transition(
        &self,
        state: &Self::State,
        event: Event,
    ) -> Step<Self::State, Event, Self::Effect> {
        match event {
            Event::Preview(preview) => match *state {
                RangePick::Idle => Step::stay(),
                RangePick::Picking { anchor, .. } => {
                    Step::to(RangePick::Picking { anchor, preview })
                }
            },
            Event::ExternalValueChanged(_) => Step::to(RangePick::Idle),
            Event::Choose { day, .. } => match *state {
                RangePick::Idle => Step::to(RangePick::Picking {
                    anchor: day,
                    preview: None,
                })
                .with(Effect::Started(day)),
                RangePick::Picking { anchor, .. } => {
                    Step::to(RangePick::Idle).with(Effect::Commit(DateRange::new(anchor, day)))
                }
            },
            Event::Cancel(_) => Step::to(RangePick::Idle).with(Effect::Cancelled),
        }
    }
}
