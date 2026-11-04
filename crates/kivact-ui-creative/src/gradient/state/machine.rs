//! Role: Decide edit state transitions.
use super::data::{GradientOwner, Snapshot};
use kivact::state::Step;
use kivact::view::EditPhase;
#[derive(Clone, PartialEq)]
pub(in crate::gradient) enum State {
    Idle,
    Editing {
        owner: GradientOwner,
        snapshot: Snapshot,
    },
}
pub(in crate::gradient) struct Machine {
    pub(in crate::gradient) blocked: bool,
}
pub(in crate::gradient) enum Event {
    Begin {
        owner: GradientOwner,
        snapshot: Snapshot,
    },
    Commit(GradientOwner),
    Cancel,
    Blocked(bool),
    Focused(bool),
}
pub(in crate::gradient) enum Effect {
    Phase(EditPhase),
    Rollback(Snapshot),
    StopDrag,
}
impl kivact::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match (state, event) {
            (State::Idle, Event::Begin { owner, snapshot }) if !self.blocked => {
                Step::to(State::Editing { owner, snapshot }).with(Effect::Phase(EditPhase::Begin))
            }
            (State::Editing { owner, .. }, Event::Commit(by)) if *owner == by => {
                Step::to(State::Idle).with(Effect::Phase(EditPhase::Commit))
            }
            (
                State::Editing { snapshot, .. },
                Event::Cancel | Event::Blocked(true) | Event::Focused(false),
            ) => Step::to(State::Idle)
                .with(Effect::StopDrag)
                .with(Effect::Rollback(snapshot.clone()))
                .with(Effect::Phase(EditPhase::Cancel)),
            _ => Step::stay(),
        }
    }
}
