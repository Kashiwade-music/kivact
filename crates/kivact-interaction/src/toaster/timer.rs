//! Role: Pause and resume one toast deadline independently of other toasts.
use kivact_core::state::{Machine as MachineTrait, Step};
use std::time::Duration;
#[derive(Clone, PartialEq)]
pub(super) enum State {
    Running { deadline_ns: u64 },
    Paused { left_ns: u64 },
    Expired,
}
pub(super) enum Event {
    Engaged(bool),
    Expire,
}
pub(super) enum Effect {
    Dismiss,
}
pub(super) struct Machine {
    pub now_ns: u64,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match (state, event) {
            (State::Running { deadline_ns }, Event::Engaged(true)) => Step::to(State::Paused {
                left_ns: deadline_ns.saturating_sub(self.now_ns),
            })
            .cancel(Event::Expire),
            (State::Paused { left_ns }, Event::Engaged(false)) => Step::to(State::Running {
                deadline_ns: self.now_ns.saturating_add(*left_ns),
            })
            .after(Duration::from_nanos(*left_ns), Event::Expire),
            (State::Running { .. }, Event::Expire) => {
                Step::to(State::Expired).with(Effect::Dismiss)
            }
            _ => Step::stay(),
        }
    }
}
