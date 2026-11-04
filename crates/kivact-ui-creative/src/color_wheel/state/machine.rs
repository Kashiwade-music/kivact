//! Role: Decide edit state transitions.
use super::super::geometry::Channel;
use super::data::{ColorOwner, Snapshot};
use kivact::state::Step;
use kivact::view::EditPhase;
#[derive(Clone, PartialEq)]
pub(in crate::color_wheel) enum State {
    Idle,
    Editing {
        owner: ColorOwner,
        snapshot: Snapshot,
    },
}
pub(in crate::color_wheel) struct Machine {
    pub(in crate::color_wheel) blocked: bool,
}
pub(in crate::color_wheel) enum Event {
    Begin {
        owner: ColorOwner,
        snapshot: Snapshot,
    },
    Commit(ColorOwner),
    Cancel,
    Blocked(bool),
    Focused(bool),
    Alpha {
        shown: bool,
        channel: Channel,
    },
}
pub(in crate::color_wheel) enum Effect {
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
            (
                State::Editing { owner, snapshot },
                Event::Alpha {
                    shown: false,
                    channel,
                },
            ) if channel == Channel::Alpha
                || matches!(owner, ColorOwner::Field { index: 3, .. }) =>
            {
                Step::to(State::Idle)
                    .with(Effect::StopDrag)
                    .with(Effect::Rollback(snapshot.clone()))
                    .with(Effect::Phase(EditPhase::Cancel))
            }
            _ => Step::stay(),
        }
    }
}
