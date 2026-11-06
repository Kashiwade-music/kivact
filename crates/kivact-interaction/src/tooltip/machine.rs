//! Role: Track tooltip precedence and the named skip-delay interval.
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_types::cell::Generation;
pub(super) type TooltipId = Generation;
#[derive(Clone, PartialEq)]
pub(super) struct State {
    pub stack: Vec<TooltipId>,
    pub warmth: Warmth,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Warmth {
    Cold,
    Warm { until_ns: u64, excluded: TooltipId },
}
pub(super) enum Event {
    Want(TooltipId),
    Release { id: TooltipId, until_ns: u64 },
}
pub(super) enum Effect {}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let mut next = state.clone();
        match event {
            Event::Want(id) => {
                next.stack.retain(|other| *other != id);
                next.stack.push(id);
            }
            Event::Release { id, until_ns } => {
                if !next.stack.contains(&id) {
                    return Step::stay();
                }
                if next.stack.last() == Some(&id) {
                    next.warmth = Warmth::Warm {
                        until_ns,
                        excluded: id,
                    };
                }
                next.stack.retain(|other| *other != id);
            }
        }
        Step::to(next)
    }
}
