//! Role: Select opening, surface and shared navigation state transitions.
use crate::list::typeahead::Typeahead;
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::Key;
use std::marker::PhantomData;

#[derive(Clone, PartialEq)]
pub(super) enum OpenedBy {
    Pointer,
    Key(Key),
    #[allow(
        dead_code,
        reason = "Reserved opening origin required by the select state model; closed typeahead currently selects without opening."
    )]
    Typeahead,
}
pub(crate) use crate::surface::Surface;
#[derive(Clone, PartialEq)]
pub(super) enum State<K> {
    Closed {
        typeahead: Typeahead,
    },
    Open {
        highlight: Option<K>,
        hovered: Option<K>,
        opened_by: OpenedBy,
        surface: Surface,
        typeahead: Typeahead,
    },
}
impl<K> State<K> {
    pub(super) fn typeahead(&self) -> &Typeahead {
        match self {
            Self::Closed { typeahead } | Self::Open { typeahead, .. } => typeahead,
        }
    }
}
pub(super) enum Event<K> {
    Open(OpenedBy),
    OpenSheet,
    Close,
    Hover(Option<K>),
    Highlight(Option<K>),
    Typeahead(Typeahead),
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
            Event::Open(opened_by) if matches!(state, State::Closed { .. }) => {
                next = State::Open {
                    highlight: None,
                    hovered: None,
                    opened_by,
                    surface: Surface::Popover,
                    typeahead: state.typeahead().clone(),
                }
            }
            Event::OpenSheet => {
                next = State::Open {
                    highlight: None,
                    hovered: None,
                    opened_by: OpenedBy::Pointer,
                    surface: Surface::Sheet,
                    typeahead: state.typeahead().clone(),
                }
            }
            Event::Close => {
                next = State::Closed {
                    typeahead: state.typeahead().clone(),
                }
            }
            Event::Hover(key) => {
                if let State::Open { hovered, .. } = &mut next {
                    *hovered = key;
                }
            }
            Event::Highlight(key) => {
                if let State::Open { highlight, .. } = &mut next {
                    *highlight = key;
                }
            }
            Event::Typeahead(typed) => match &mut next {
                State::Closed { typeahead } | State::Open { typeahead, .. } => *typeahead = typed,
            },
            _ => return Step::stay(),
        }
        Step::to(next)
    }
}
