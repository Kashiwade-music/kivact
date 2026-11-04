//! Role: Decide edit state transitions.
use super::super::data::CubicBezier;
use kivact::state::Step;
use kivact::view::EditPhase;
pub(in crate::bezier) struct Machine;
#[derive(Clone, Copy, PartialEq)]
pub(in crate::bezier) enum State {
    Idle,
    Editing { initial: CubicBezier },
}
pub(in crate::bezier) enum Event {
    Begin(CubicBezier),
    Commit,
    Cancel,
    InputChanged(bool),
}
pub(in crate::bezier) enum Effect {
    Restore(CubicBezier),
    StopDrag,
    Phase(EditPhase),
}
impl kivact::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match (state, event) {
            (State::Idle, Event::Begin(initial)) => {
                Step::to(State::Editing { initial }).with(Effect::Phase(EditPhase::Begin))
            }
            (State::Editing { .. }, Event::Commit) => {
                Step::to(State::Idle).with(Effect::Phase(EditPhase::Commit))
            }
            (State::Editing { initial }, Event::Cancel | Event::InputChanged(true)) => {
                Step::to(State::Idle)
                    .with(Effect::StopDrag)
                    .with(Effect::Restore(*initial))
                    .with(Effect::Phase(EditPhase::Cancel))
            }
            _ => Step::stay(),
        }
    }
}
