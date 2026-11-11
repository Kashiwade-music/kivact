//! Role: Select opening, surface and shared navigation state transitions.
use crate::list::typeahead::{TYPEAHEAD_WINDOW, Typeahead};
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::Key;
use std::marker::PhantomData;

#[derive(Clone, PartialEq)]
pub(super) enum OpenedBy {
    Pointer,
    Key(Key),
}
pub(crate) use crate::surface::Surface;
#[derive(Clone, PartialEq)]
pub(super) enum State<K> {
    Closed {
        typeahead: Typeahead,
    },
    Open {
        // xtask-allow(state-fields): independent: the key the keyboard highlights in the open list
        highlight: Option<K>,
        // xtask-allow(state-fields): independent: the key the pointer is over in the open list
        hovered: Option<K>,
        // xtask-allow(state-fields): independent: the option last reported to the preview callback (a record of a call already made, not derivable from hovered/highlight, which change before track_preview reports); Open and OpenSheet reset it to None, Event::Preview sets it, Close reports None when it held a key
        /// The option last reported to the caller's preview callback.
        previewed: Option<K>,
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
    /// The option to report to the caller's preview callback.
    Preview(Option<K>),
    Typeahead(Typeahead),
    /// The type-to-find window ended.
    Expire,
}
pub(super) enum Effect<K> {
    Preview(Option<K>),
}
pub(super) struct Machine<K>(PhantomData<fn() -> K>);
impl<K> Machine<K> {
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}
/// The open state of a list shown as a sheet, with nothing highlighted or previewed.
fn sheet_state<K: Clone>(state: &State<K>) -> State<K> {
    State::Open {
        highlight: None,
        hovered: None,
        previewed: None,
        opened_by: OpenedBy::Pointer,
        surface: Surface::Sheet,
        typeahead: state.typeahead().clone(),
    }
}
impl<K: Clone + PartialEq + 'static> MachineTrait for Machine<K> {
    type State = State<K>;
    type Event = Event<K>;
    type Effect = Effect<K>;
    fn transition(&self, state: &State<K>, event: Event<K>) -> Step<State<K>, Event<K>, Effect<K>> {
        let mut next = state.clone();
        match event {
            Event::Open(opened_by) if matches!(state, State::Closed { .. }) => {
                next = State::Open {
                    highlight: None,
                    hovered: None,
                    previewed: None,
                    opened_by,
                    surface: Surface::Popover,
                    typeahead: state.typeahead().clone(),
                }
            }
            Event::OpenSheet => {
                let reported = matches!(
                    state,
                    State::Open {
                        previewed: Some(_),
                        ..
                    }
                );
                next = sheet_state(state);
                if reported {
                    return Step::to(next).with(Effect::Preview(None));
                }
            }
            Event::Close => {
                let previewed = match state {
                    State::Open { previewed, .. } => previewed.is_some(),
                    State::Closed { .. } => false,
                };
                next = State::Closed {
                    typeahead: state.typeahead().clone(),
                };
                if previewed {
                    return Step::to(next).with(Effect::Preview(None));
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
            Event::Preview(key) => {
                let changed = match state {
                    State::Open { previewed, .. } => *previewed != key,
                    State::Closed { .. } => false,
                };
                if let State::Open { previewed, .. } = &mut next {
                    *previewed = key.clone();
                }
                if changed {
                    return Step::to(next).with(Effect::Preview(key));
                }
            }
            Event::Typeahead(typed) => {
                let typing = matches!(typed, Typeahead::Typing { .. });
                match &mut next {
                    State::Closed { typeahead } | State::Open { typeahead, .. } => {
                        *typeahead = typed
                    }
                }
                if typing {
                    return Step::to(next).after(TYPEAHEAD_WINDOW, Event::Expire);
                }
            }
            Event::Expire => match &mut next {
                State::Closed { typeahead } | State::Open { typeahead, .. } => {
                    *typeahead = Typeahead::Idle
                }
            },
            _ => return Step::stay(),
        }
        Step::to(next)
    }
}
