//! Role: Decide edit state transitions.
use kivact::state::Step;
pub(crate) struct Machine;
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum State {
    Idle,
    Pressed,
    Moved,
}
pub(crate) enum Event {
    Press,
    Move,
    End,
}
impl kivact::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = ();
    fn transition(&self, _: &State, event: Event) -> Step<State, Event, ()> {
        Step::to(match event {
            Event::Press => State::Pressed,
            Event::Move => State::Moved,
            Event::End => State::Idle,
        })
    }
}
