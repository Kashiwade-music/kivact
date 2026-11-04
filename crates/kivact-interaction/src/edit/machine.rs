//! Role: Pure edit-session transitions shared with draft editing.
use kivact_core::state::{Machine as MachineTrait, Step};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditOrigin {
    Programmatic,
}

#[derive(Clone, PartialEq)]
pub(crate) enum EditSessionState<T> {
    Idle,
    Editing { initial: T, origin: EditOrigin },
}
pub(super) struct Initial<T>(Rc<T>);
impl<T> Initial<T> {
    pub(super) fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}
impl<T> Clone for Initial<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> PartialEq for Initial<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
pub(super) struct Machine<T>(PhantomData<fn() -> T>);
impl<T> Machine<T> {
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}
pub(super) enum Event<T> {
    Begin(Rc<T>, EditOrigin),
    Finish,
}
impl<T: 'static> MachineTrait for Machine<T> {
    type State = EditSessionState<Initial<T>>;
    type Event = Event<T>;
    type Effect = ();
    fn transition(&self, state: &Self::State, event: Event<T>) -> Step<Self::State, Event<T>, ()> {
        match (state, event) {
            (EditSessionState::Idle, Event::Begin(initial, origin)) => {
                Step::to(EditSessionState::Editing {
                    initial: Initial(initial),
                    origin,
                })
            }
            (EditSessionState::Editing { .. }, Event::Finish) => Step::to(EditSessionState::Idle),
            _ => Step::stay(),
        }
    }
}
