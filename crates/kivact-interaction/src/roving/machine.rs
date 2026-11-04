//! Role: Serialize requested, observed and removed-member focus events.
use kivact_core::custom::NodeToken;
use kivact_core::state::{Machine as MachineTrait, Step};

#[derive(Clone, PartialEq)]
pub(super) enum Roving<Id> {
    Unset,
    Requested {
        key: Id,
        observed: Option<NodeToken>,
    },
    Observed(NodeToken),
}
impl<Id> Roving<Id> {
    pub(super) fn observed(&self) -> Option<NodeToken> {
        match self {
            Self::Unset => None,
            Self::Requested { observed, .. } => *observed,
            Self::Observed(token) => Some(*token),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Restore {
    Nothing,
    From(NodeToken),
}
#[derive(Clone, PartialEq)]
pub(super) struct State<Id> {
    pub(super) roving: Roving<Id>,
    pub(super) restore: Restore,
}
pub(super) enum Event<Id> {
    Request(Option<Id>),
    Share,
    Observe(Option<NodeToken>),
    Restore(Option<NodeToken>),
}
pub(super) enum Effect<Id> {
    Request(Option<Id>),
}
pub(super) struct Machine<Id> {
    shared: bool,
    marker: std::marker::PhantomData<fn() -> Id>,
}
impl<Id> Machine<Id> {
    pub(super) fn new(shared: bool) -> Self {
        Self {
            shared,
            marker: std::marker::PhantomData,
        }
    }
}
impl<Id: Clone + PartialEq + 'static> MachineTrait for Machine<Id> {
    type State = State<Id>;
    type Event = Event<Id>;
    type Effect = Effect<Id>;
    fn transition(
        &self,
        state: &State<Id>,
        event: Event<Id>,
    ) -> Step<State<Id>, Event<Id>, Effect<Id>> {
        let mut next = state.clone();
        match event {
            Event::Request(key) if self.shared => Step::stay().with(Effect::Request(key)),
            Event::Request(key) => {
                next.roving = match &key {
                    Some(key) => Roving::Requested {
                        key: key.clone(),
                        observed: state.roving.observed(),
                    },
                    None => state
                        .roving
                        .observed()
                        .map(Roving::Observed)
                        .unwrap_or(Roving::Unset),
                };
                Step::to(next)
            }
            Event::Share => {
                next.roving = state
                    .roving
                    .observed()
                    .map(Roving::Observed)
                    .unwrap_or(Roving::Unset);
                Step::to(next)
            }
            Event::Observe(token) => {
                next.roving = match &state.roving {
                    Roving::Requested { key, .. } => Roving::Requested {
                        key: key.clone(),
                        observed: token,
                    },
                    _ => token.map(Roving::Observed).unwrap_or(Roving::Unset),
                };
                Step::to(next)
            }
            Event::Restore(token) => {
                next.restore = token.map(Restore::From).unwrap_or(Restore::Nothing);
                Step::to(next)
            }
        }
    }
}
