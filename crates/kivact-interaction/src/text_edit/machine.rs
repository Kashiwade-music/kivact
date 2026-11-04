//! Role: Control mounting and IME composition transitions.
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_types::cell::Generation;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Composition {
    Idle,
    Composing,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Unmounted,
    Mounted {
        id: Generation,
        composition: Composition,
    },
}
pub(super) enum Event {
    Mount(Generation),
    Unmount(Generation),
    Composition { id: Generation, composing: bool },
}
pub(super) enum Effect {
    DuplicateMount,
    Composition(bool),
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        match (*state, event) {
            (State::Unmounted, Event::Mount(id)) => Step::to(State::Mounted {
                id,
                composition: Composition::Idle,
            }),
            (State::Mounted { .. }, Event::Mount(_)) => Step::stay().with(Effect::DuplicateMount),
            (State::Mounted { id, .. }, Event::Unmount(current)) if id == current => {
                Step::to(State::Unmounted)
            }
            (
                State::Mounted { id, composition },
                Event::Composition {
                    id: current,
                    composing,
                },
            ) if id == current => {
                let next = if composing {
                    Composition::Composing
                } else {
                    Composition::Idle
                };
                if next == composition {
                    Step::stay()
                } else {
                    Step::to(State::Mounted {
                        id,
                        composition: next,
                    })
                    .with(Effect::Composition(composing))
                }
            }
            _ => Step::stay(),
        }
    }
}
