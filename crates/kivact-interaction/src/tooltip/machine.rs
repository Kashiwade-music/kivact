//! Role: Track tooltip precedence and the named skip-delay interval.
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_types::cell::Generation;
use std::time::Duration;
pub(super) type TooltipId = Generation;
#[derive(Clone, PartialEq)]
pub(super) struct State {
    pub stack: Vec<TooltipId>,
    pub warmth: Warmth,
}
/// Whether the next tooltip skips its delay: for `within` after one hid.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Warmth {
    Cold,
    Warm { excluded: TooltipId },
}
pub(super) enum Event {
    Want(TooltipId),
    Release {
        id: TooltipId,
        within: Duration,
    },
    /// The skip-delay interval of the last release ended.
    Cool,
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
            Event::Release { id, within } => {
                if !next.stack.contains(&id) {
                    return Step::stay();
                }
                let warm = next.stack.last() == Some(&id);
                if warm {
                    next.warmth = Warmth::Warm { excluded: id };
                }
                next.stack.retain(|other| *other != id);
                let mut step = Step::to(next);
                if warm {
                    step = step.after(within, Event::Cool);
                }
                return step;
            }
            Event::Cool => next.warmth = Warmth::Cold,
        }
        Step::to(next)
    }
}
