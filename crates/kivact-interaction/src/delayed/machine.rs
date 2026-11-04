//! Role: Compute delayed visibility transitions and own their timer operations.
use kivact_core::state::{Machine as MachineTrait, Step};
use std::time::Duration;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Hidden,
    Opening,
    Shown,
    Closing,
    Dismissed,
}
pub(super) enum Event {
    Wanted(bool),
    Dismiss,
    Pulse,
    OpenDue,
    CloseDue,
}
pub(super) enum Effect {}
pub(super) struct Machine {
    pub wanted: bool,
    pub open: Duration,
    pub close: Duration,
}
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match event {
            Event::Wanted(true) => match state {
                State::Hidden if self.open.is_zero() => Step::to(State::Shown),
                State::Hidden => Step::to(State::Opening).after(self.open, Event::OpenDue),
                State::Closing => Step::to(State::Shown).cancel(Event::CloseDue),
                _ => Step::stay(),
            },
            Event::Wanted(false) => match state {
                State::Opening => Step::to(State::Hidden).cancel(Event::OpenDue),
                State::Shown => self.close(),
                State::Dismissed => Step::to(State::Hidden),
                _ => Step::stay(),
            },
            Event::Dismiss => Step::to(if self.wanted {
                State::Dismissed
            } else {
                State::Hidden
            })
            .cancel(Event::OpenDue)
            .cancel(Event::CloseDue),
            Event::Pulse => {
                let step = if self.wanted {
                    Step::to(State::Shown).cancel(Event::CloseDue)
                } else {
                    self.close()
                };
                step.cancel(Event::OpenDue)
            }
            Event::OpenDue if *state == State::Opening => Step::to(State::Shown),
            Event::CloseDue if *state == State::Closing => Step::to(State::Hidden),
            Event::OpenDue | Event::CloseDue => Step::stay(),
        }
    }
}
impl Machine {
    fn close(&self) -> Step<State, Event, Effect> {
        if self.close.is_zero() {
            Step::to(State::Hidden).cancel(Event::CloseDue)
        } else {
            Step::to(State::Closing).after(self.close, Event::CloseDue)
        }
    }
}
