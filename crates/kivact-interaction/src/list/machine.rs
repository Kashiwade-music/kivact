//! Role: Keep virtual focus requests until their source loses focus.
use kivact_core::custom::{NodeRefExt, NodeToken};
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::NodeRef;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub(super) struct Origin {
    pub(super) token: Option<NodeToken>,
    // xtask-allow(state-fields): A snapshot reference locates the origin token through virtualization.
    pub(super) node: Option<NodeRef>,
}
impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
            && match (self.node, other.node) {
                (Some(left), Some(right)) => left.same_ref(right),
                (None, None) => true,
                _ => false,
            }
    }
}
impl Origin {
    pub(super) fn kept(self, current: Option<NodeToken>) -> bool {
        current == self.token
            || (current.is_none()
                && self.token.is_some()
                && self.node.is_some_and(|node| node.mounted_node().is_none()))
    }
}
#[derive(Clone, PartialEq)]
pub(super) enum Pending<K> {
    Nothing,
    Activating { key: K, origin: Origin },
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Focus {
    Physical,
    Retained,
}

#[derive(Clone, PartialEq)]
pub(super) struct State<K> {
    pub(super) pending: Pending<K>,
    pub(super) position: usize,
    pub(super) focus: Focus,
}
pub(super) enum Event<K> {
    Activate { key: K, origin: Origin },
    Cancel,
    Position(usize),
    MountFocus(Focus),
    Observe(Option<NodeToken>),
}
pub(super) struct Machine<K>(PhantomData<fn() -> K>);
impl<K> Machine<K> {
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}
impl<K: Clone + PartialEq + 'static> MachineTrait for Machine<K> {
    type State = State<K>;
    type Event = Event<K>;
    type Effect = ();
    fn transition(&self, state: &State<K>, event: Event<K>) -> Step<State<K>, Event<K>, ()> {
        let mut next = state.clone();
        match event {
            Event::Activate { key, origin } => next.pending = Pending::Activating { key, origin },
            Event::Cancel => next.pending = Pending::Nothing,
            Event::Position(position) => next.position = position,
            Event::MountFocus(focus) => next.focus = focus,
            Event::Observe(current) => match state.pending {
                Pending::Activating { origin, .. } if !origin.kept(current) => {
                    next.pending = Pending::Nothing
                }
                _ => {}
            },
        }
        Step::to(next)
    }
}
