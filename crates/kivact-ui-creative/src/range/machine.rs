//! Role: Decide range endpoint transitions.
use super::data::RangePart;
use kivact::state::Step;
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct State {
    pub(crate) active: RangePart,
    pub(crate) value: (f64, f64),
}
pub(crate) struct Machine;
pub(crate) enum Event {
    Press { part: RangePart, pressed: bool },
    Select(RangePart),
    ValueChanged { before: (f64, f64), now: (f64, f64) },
}
impl kivact::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = ();
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, ()> {
        match event {
            Event::Select(active)
            | Event::Press {
                part: active,
                pressed: true,
            } => Step::to(State { active, ..*state }),
            Event::Press { pressed: false, .. } => Step::stay(),
            Event::ValueChanged { before, now } => {
                let active = if before.0 != now.0 && before.1 == now.1 {
                    RangePart::Start
                } else if before.0 == now.0 && before.1 != now.1 {
                    RangePart::End
                } else {
                    state.active
                };
                Step::to(State { active, value: now })
            }
        }
    }
}
