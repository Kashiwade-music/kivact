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
    /// The owner's open request changed: `false` dismisses, `true` changes nothing.
    Request(bool),
    /// The surface may open at once (focus arrived): while `Opening`, acts as `Pulse`.
    Immediate(bool),
}
/// A change of the visibility, reported to the owner.
pub(super) enum Effect {
    /// Entered `Shown` from another state.
    Shown,
    /// Entered `Hidden` or `Dismissed` from another state.
    Hidden,
}
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
        let from = *state;
        match event {
            Event::Wanted(true) => match from {
                State::Hidden if self.open.is_zero() => enter(from, State::Shown),
                State::Hidden => enter(from, State::Opening).after(self.open, Event::OpenDue),
                State::Closing => enter(from, State::Shown).cancel(Event::CloseDue),
                _ => Step::stay(),
            },
            Event::Wanted(false) => match from {
                State::Opening => enter(from, State::Hidden).cancel(Event::OpenDue),
                State::Shown => self.close(from),
                State::Dismissed => enter(from, State::Hidden),
                _ => Step::stay(),
            },
            Event::Dismiss | Event::Request(false) => self.dismiss(from),
            Event::Request(true) => Step::stay(),
            Event::Pulse => self.pulse(from),
            Event::Immediate(true) if from == State::Opening => self.pulse(from),
            Event::Immediate(_) => Step::stay(),
            Event::OpenDue if from == State::Opening => enter(from, State::Shown),
            Event::CloseDue if from == State::Closing => enter(from, State::Hidden),
            Event::OpenDue | Event::CloseDue => Step::stay(),
        }
    }
}
impl Machine {
    fn close(&self, from: State) -> Step<State, Event, Effect> {
        if self.close.is_zero() {
            enter(from, State::Hidden).cancel(Event::CloseDue)
        } else {
            enter(from, State::Closing).after(self.close, Event::CloseDue)
        }
    }
    fn dismiss(&self, from: State) -> Step<State, Event, Effect> {
        enter(
            from,
            if self.wanted {
                State::Dismissed
            } else {
                State::Hidden
            },
        )
        .cancel(Event::OpenDue)
        .cancel(Event::CloseDue)
    }
    fn pulse(&self, from: State) -> Step<State, Event, Effect> {
        let step = if self.wanted {
            enter(from, State::Shown).cancel(Event::CloseDue)
        } else {
            self.close(from)
        };
        step.cancel(Event::OpenDue)
    }
}
/// Move to `to`, reporting an entry into `Shown` or into a hidden state.
fn enter(from: State, to: State) -> Step<State, Event, Effect> {
    let step = Step::to(to);
    if from == to {
        return step;
    }
    match to {
        State::Shown => step.with(Effect::Shown),
        State::Hidden | State::Dismissed => step.with(Effect::Hidden),
        State::Opening | State::Closing => step,
    }
}
