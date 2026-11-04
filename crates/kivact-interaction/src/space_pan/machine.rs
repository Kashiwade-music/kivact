//! Role: Represent the Space modifier and disarm on unavailable surfaces.
use kivact_core::state::{Machine as MachineTrait, Step};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpaceKey {
    Up,
    Held,
}
pub(super) enum Event {
    Key(bool),
    Disarm,
    Available(bool),
}
pub(super) enum Effect {}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = SpaceKey;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, _: &SpaceKey, event: Event) -> Step<SpaceKey, Event, Effect> {
        match event {
            Event::Key(true) => Step::to(SpaceKey::Held),
            Event::Key(false) | Event::Disarm | Event::Available(false) => Step::to(SpaceKey::Up),
            Event::Available(true) => Step::stay(),
        }
    }
}
