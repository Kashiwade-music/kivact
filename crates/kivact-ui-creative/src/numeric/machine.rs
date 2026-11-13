//! Role: Decide edit state transitions and own the scrub anchor.

use kivact::state::Step;

pub(crate) struct Machine;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum State {
    Idle,
    Scrubbing { moved: bool, anchor: Anchor },
    Typing,
}

/// Where a scrub measures from: the pointer travel along the drag axis, the
/// value or position there, the modifier factor in force, and which space
/// `origin` is in.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Anchor {
    pub(crate) along: f64,
    pub(crate) origin: f64,
    pub(crate) factor: f64,
    pub(crate) space: ScrubSpace,
}

/// The space an anchor's `origin` is measured in.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ScrubSpace {
    Value,
    Position,
}

pub(crate) enum Event {
    Press(Anchor),
    Move,
    Reanchor(Anchor),
    End,
    BeginText,
    EndText,
}

impl kivact::state::Machine for Machine {
    type State = State;
    type Event = Event;
    type Effect = ();
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, ()> {
        match event {
            Event::Press(anchor) => Step::to(State::Scrubbing {
                moved: false,
                anchor,
            }),
            Event::Move => match state {
                State::Scrubbing { anchor, .. } => Step::to(State::Scrubbing {
                    moved: true,
                    anchor: *anchor,
                }),
                _ => Step::stay(),
            },
            Event::Reanchor(anchor) => match state {
                State::Scrubbing { moved, .. } => Step::to(State::Scrubbing {
                    moved: *moved,
                    anchor,
                }),
                _ => Step::stay(),
            },
            Event::End => match state {
                State::Scrubbing { .. } => Step::to(State::Idle),
                _ => Step::stay(),
            },
            Event::BeginText => Step::to(State::Typing),
            Event::EndText => Step::to(State::Idle),
        }
    }
}
