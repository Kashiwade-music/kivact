//! Role: Decide edit state transitions.
use kivact::state::Step;
pub(crate) struct Machine;
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum State {
    Idle,
    Scrubbing { moved: bool },
    Typing,
}
pub(crate) enum Event {
    Press,
    Move,
    End,
    BeginText,
    EndText,
}
impl kivact::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = ();
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, ()> {
        match event {
            Event::Press => Step::to(State::Scrubbing { moved: false }),
            Event::Move => match state {
                State::Scrubbing { .. } => Step::to(State::Scrubbing { moved: true }),
                _ => Step::stay(),
            },
            Event::End => match state {
                State::Scrubbing { .. } => Step::to(State::Idle),
                _ => Step::stay(),
            },
            Event::BeginText => Step::to(State::Typing),
            Event::EndText => Step::to(State::Idle),
        }
    }
}
