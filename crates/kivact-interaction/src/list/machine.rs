//! Role: Keep virtual focus requests until their source loses focus.
use super::typeahead::{TYPEAHEAD_WINDOW, Typeahead};
use kivact_core::custom::{NodeRefExt, NodeToken};
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::NodeRef;
use std::marker::PhantomData;

/// Where the focus was when a virtual focus request was made.
#[derive(Clone, Copy)]
pub(super) enum Origin {
    /// No node had the focus.
    Nowhere,
    /// A node that is not a member of the list had the focus.
    Outside(NodeToken),
    /// A member had the focus; its reference finds it again when
    /// virtualization unmounts it.
    Member { token: NodeToken, node: NodeRef },
}
impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nowhere, Self::Nowhere) => true,
            (Self::Outside(left), Self::Outside(right)) => left == right,
            (
                Self::Member {
                    token: left,
                    node: a,
                },
                Self::Member {
                    token: right,
                    node: b,
                },
            ) => left == right && a.same_ref(*b),
            _ => false,
        }
    }
}
impl Origin {
    pub(super) fn kept(self, current: Option<NodeToken>) -> bool {
        match self {
            Self::Nowhere => current.is_none(),
            Self::Outside(token) => current == Some(token),
            Self::Member { token, node } => {
                current == Some(token) || (current.is_none() && node.mounted_node().is_none())
            }
        }
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
    pub(super) focus: Focus,
    pub(super) typed: Typeahead,
}
pub(super) enum Event<K> {
    Activate {
        key: K,
        origin: Origin,
    },
    Cancel,
    MountFocus(Focus),
    Observe(Option<NodeToken>),
    Typeahead(Typeahead),
    /// The type-to-find window ended.
    Expire,
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
            Event::MountFocus(focus) => next.focus = focus,
            Event::Observe(current) => match state.pending {
                Pending::Activating { origin, .. } if !origin.kept(current) => {
                    next.pending = Pending::Nothing
                }
                _ => {}
            },
            Event::Typeahead(typed) => {
                if matches!(typed, Typeahead::Typing { .. }) {
                    next.typed = typed;
                    return Step::to(next).after(TYPEAHEAD_WINDOW, Event::Expire);
                }
                next.typed = typed;
            }
            Event::Expire => next.typed = Typeahead::Idle,
        }
        Step::to(next)
    }
}
