//! Role: Own the active date-time picker panel and accepted endpoint bindings.
use kivact_core::i18n::{CalendarDate, DateTime, TimeOfDay};
use kivact_core::state::{Binding, Machine, Step, Value, binding};
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Panel {
    Date,
    Time,
}
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DateTimeInputState {
    Closed,
    Open {
        panel: Panel,
        invoker: kivact_core::view::NodeRef,
    },
}
pub(crate) enum Event {
    Open(Panel, kivact_core::view::NodeRef),
    Switch(Panel),
    Close,
}
pub(crate) struct Controller;
impl Machine for Controller {
    type State = DateTimeInputState;
    type Event = Event;
    type Effect = ();
    fn transition(
        &self,
        state: &DateTimeInputState,
        event: Event,
    ) -> Step<DateTimeInputState, Event, ()> {
        Step::to(match event {
            Event::Open(panel, invoker) => DateTimeInputState::Open { panel, invoker },
            Event::Switch(panel) => match *state {
                DateTimeInputState::Open { invoker, .. } => {
                    DateTimeInputState::Open { panel, invoker }
                }
                DateTimeInputState::Closed => DateTimeInputState::Closed,
            },
            Event::Close => DateTimeInputState::Closed,
        })
    }
}
fn key(value: DateTime) -> (CalendarDate, u8, u8, u8) {
    (
        value.date,
        value.time.hour,
        value.time.minute,
        value.time.second,
    )
}
/// Endpoint bindings project the caller-owned accepted date/time value.
pub(crate) struct DateTimeFields {
    pub(crate) date: Binding<Option<CalendarDate>>,
    pub(crate) time: Binding<Option<TimeOfDay>>,
}
/// Preserve the complementary endpoint while validating each accepted write.
pub(crate) fn field_bindings(
    value: Binding<Option<DateTime>>,
    min: Value<Option<DateTime>>,
    max: Value<Option<DateTime>>,
) -> DateTimeFields {
    let accepted = binding(value, move |next: Option<DateTime>| {
        if next.is_none_or(|next| {
            min.get().is_none_or(|min| key(next) >= key(min))
                && max.get().is_none_or(|max| key(next) <= key(max))
        }) {
            value.set(next);
        }
    });
    let date = binding(
        move || value.get().map(|value| value.date),
        move |next| {
            accepted.set(next.map(|date| DateTime {
                date,
                time: value.get_untracked().map_or(
                    TimeOfDay {
                        hour: 0,
                        minute: 0,
                        second: 0,
                    },
                    |value| value.time,
                ),
            }))
        },
    );
    let time = binding(
        move || value.get().map(|value| value.time),
        move |next| {
            accepted.set(next.map(|time| {
                DateTime {
                    time,
                    date: value
                        .get_untracked()
                        .map_or_else(CalendarDate::today, |value| value.date),
                }
            }))
        },
    );
    DateTimeFields { date, time }
}
