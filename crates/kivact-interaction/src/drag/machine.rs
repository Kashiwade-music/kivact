//! Role: Pure ownership and lifecycle transitions for one pointer drag.
use super::Active;
use kivact_core::geometry::Point;
use kivact_core::state::{Machine as MachineTrait, Step};
use std::marker::PhantomData;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EditOwnership {
    Owned,
    Borrowed,
}
pub(super) enum State<T> {
    Idle,
    Active(Active<T>),
    Releasing { edit: EditOwnership },
}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Active(a) => Self::Active(a.clone()),
            Self::Releasing { edit } => Self::Releasing { edit: *edit },
        }
    }
}
impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Idle, Self::Idle) => true,
            (Self::Active(a), Self::Active(b)) => {
                a.source == b.source
                    && a.last == b.last
                    && a.edit == b.edit
                    && std::rc::Rc::ptr_eq(&a.start, &b.start)
            }
            (Self::Releasing { edit: a }, Self::Releasing { edit: b }) => a == b,
            _ => false,
        }
    }
}
pub(super) struct Machine<T>(PhantomData<fn() -> T>);
impl<T> Machine<T> {
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}
pub(super) enum Event<T> {
    Begin(Active<T>),
    Position(Point),
    OwnEdit,
    Release(EditOwnership),
    Stop,
    Disabled(bool),
}
pub(super) enum Effect {
    Commit,
}
impl<T: 'static> MachineTrait for Machine<T> {
    type State = State<T>;
    type Event = Event<T>;
    type Effect = Effect;
    fn transition(&self, state: &State<T>, event: Event<T>) -> Step<State<T>, Event<T>, Effect> {
        match (state, event) {
            (State::Idle, Event::Begin(active)) => Step::to(State::Active(active)),
            (State::Active(active), Event::Position(pos)) => {
                let mut active = active.clone();
                active.last = pos;
                Step::to(State::Active(active))
            }
            (State::Active(active), Event::OwnEdit) => {
                let mut active = active.clone();
                active.edit = EditOwnership::Owned;
                Step::to(State::Active(active))
            }
            (State::Releasing { .. }, Event::OwnEdit) => Step::to(State::Releasing {
                edit: EditOwnership::Owned,
            }),
            (_, Event::Release(edit)) => Step::to(State::Releasing { edit }),
            (_, Event::Stop) => Step::to(State::Idle),
            (State::Active(active), Event::Disabled(true)) => {
                let step = Step::to(State::Idle);
                if active.edit == EditOwnership::Owned {
                    step.with(Effect::Commit)
                } else {
                    step
                }
            }
            _ => Step::stay(),
        }
    }
}
