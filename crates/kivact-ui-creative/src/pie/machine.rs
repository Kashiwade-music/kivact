//! Role: Decide edit state transitions.
use super::data::{PieEntry, PieState};
use kivact::geometry::Point;
use kivact::state::Step;
pub(crate) struct Machine {
    pub(crate) entries: Vec<PieEntry>,
}
pub(crate) enum Event {
    External(PieState),
    Close,
    Choose(usize),
    Pointer(Point),
    Navigate(bool),
}
pub(crate) enum Effect {
    Publish(PieState),
    Choose(usize),
}
impl kivact::state::Machine for Machine {
    type State = PieState;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &PieState, event: Event) -> Step<PieState, Event, Effect> {
        let next = match event {
            Event::External(next) => return Step::to(next),
            Event::Close => PieState::Closed,
            Event::Choose(index) => {
                return Step::to(PieState::Closed)
                    .with(Effect::Choose(index))
                    .with(Effect::Publish(PieState::Closed));
            }
            Event::Pointer(point) => {
                let mut next = *state;
                next.pointer(point, &self.entries);
                next
            }
            Event::Navigate(forward) => {
                let mut next = *state;
                if let PieState::Open { hot, .. } = &mut next {
                    let len = self.entries.len();
                    if len != 0 {
                        let index = hot.unwrap_or(0).min(len - 1);
                        *hot = Some(if forward {
                            (index + 1) % len
                        } else {
                            (index + len - 1) % len
                        });
                    }
                }
                next
            }
        };
        Step::to(next).with(Effect::Publish(next))
    }
}
