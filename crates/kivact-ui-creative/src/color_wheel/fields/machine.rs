//! Role: Decide edit state transitions.
use super::data::HexField;
use kivact::state::Step;
pub(in crate::color_wheel) struct Machine;
pub(in crate::color_wheel) enum Event {
    Typed(String),
    Follow,
}
impl kivact::state::Machine for Machine {
    type State = HexField;
    type Event = Event;
    type Effect = ();
    fn transition(&self, _: &HexField, event: Event) -> Step<HexField, Event, ()> {
        Step::to(match event {
            Event::Typed(text) => HexField::Typed(text),
            Event::Follow => HexField::Following,
        })
    }
}
