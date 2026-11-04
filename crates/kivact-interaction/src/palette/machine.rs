//! Role: Palette visibility transitions reset its combobox atomically.
use kivact_core::state::{Machine as MachineTrait, Step};
#[derive(Clone, Copy, PartialEq)]
pub(super) enum State {
    Closed,
    Open,
}
pub(super) enum Event {
    Close,
    Open,
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = ();
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, ()> {
        match (state, event) {
            (State::Open, Event::Close) => Step::to(State::Closed).with(()),
            (State::Closed, Event::Open) => Step::to(State::Open),
            _ => Step::stay(),
        }
    }
}
