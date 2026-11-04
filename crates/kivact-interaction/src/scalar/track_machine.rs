//! Role: Keep track attachment geometry and the current grab in one phase.
use super::track_data::{Geometry, Grab};
use kivact_core::state::{Machine as MachineTrait, Step};
#[derive(Clone, Copy, PartialEq)]
pub(super) enum TrackGesture {
    Idle,
    Grabbing(Grab),
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum State {
    Detached,
    Attached {
        geometry: Geometry,
        gesture: TrackGesture,
    },
}
pub(super) enum Event {
    Attach(Geometry),
    Grab(Grab),
    Stop,
}
pub(super) enum Effect {}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match (state, event) {
            (State::Attached { gesture, .. }, Event::Attach(geometry)) => {
                Step::to(State::Attached {
                    geometry,
                    gesture: *gesture,
                })
            }
            (State::Detached, Event::Attach(geometry)) => Step::to(State::Attached {
                geometry,
                gesture: TrackGesture::Idle,
            }),
            (State::Attached { geometry, .. }, Event::Grab(grab)) => Step::to(State::Attached {
                geometry: *geometry,
                gesture: TrackGesture::Grabbing(grab),
            }),
            (State::Attached { geometry, .. }, Event::Stop) => Step::to(State::Attached {
                geometry: *geometry,
                gesture: TrackGesture::Idle,
            }),
            _ => Step::stay(),
        }
    }
}
