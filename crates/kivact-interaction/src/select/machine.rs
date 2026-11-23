//! Role: Select opening, surface and shared navigation state transitions.
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::Key;
use std::marker::PhantomData;

#[derive(Clone, PartialEq)]
pub(super) enum OpenedBy {
    Pointer,
    Key(Key),
}
pub(crate) use crate::surface::Surface;

/// The details of the next opening, noted before the caller's `open` is written.
#[derive(Clone, PartialEq)]
pub(super) struct Opening {
    pub(super) opened_by: OpenedBy,
    pub(super) surface: Surface,
}
impl Default for Opening {
    fn default() -> Self {
        Self {
            opened_by: OpenedBy::Pointer,
            surface: Surface::Popover,
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) enum State<K> {
    /// Closed, holding the details the next opening takes.
    Closed { next: Opening },
    Open {
        highlight: Option<K>,
        hovered: Option<K>,
        /// The option last reported to the caller's preview callback.
        previewed: Option<K>,
        opened_by: OpenedBy,
        surface: Surface,
    },
}
impl<K> State<K> {
    /// The state of a closed list, with the default next opening.
    pub(super) fn closed() -> Self {
        Self::Closed {
            next: Opening::default(),
        }
    }
    /// The state of a list opened with `opening`, nothing highlighted or previewed.
    pub(super) fn open_with(opening: Opening) -> Self {
        Self::Open {
            highlight: None,
            hovered: None,
            previewed: None,
            opened_by: opening.opened_by,
            surface: opening.surface,
        }
    }
}
pub(super) enum Event<K> {
    /// Note the details of the next opening; an open list moves to a sheet.
    Prepare(Opening),
    /// The caller's `open` became true: the follow of `open`.
    Opened,
    /// The caller's `open` became false: the follow of `open`.
    Closed,
    /// The list is being hidden: a note noted before the hide must not outlive it.
    Closing,
    Hover(Option<K>),
    Highlight(Option<K>),
    /// The option to report to the caller's preview callback.
    Preview(Option<K>),
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
fn sheet_state<K>() -> State<K> {
    State::open_with(Opening {
        opened_by: OpenedBy::Pointer,
        surface: Surface::Sheet,
    })
}
impl<K: Clone + PartialEq + 'static> MachineTrait for Machine<K> {
    type State = State<K>;
    type Event = Event<K>;
    type Effect = Effect<K>;
    fn transition(&self, state: &State<K>, event: Event<K>) -> Step<State<K>, Event<K>, Effect<K>> {
        let mut next = state.clone();
        match event {
            Event::Prepare(opening) => match state {
                State::Closed { .. } => next = State::Closed { next: opening },
                State::Open { previewed, .. } if opening.surface == Surface::Sheet => {
                    let reported = previewed.is_some();
                    next = sheet_state();
                    if reported {
                        return Step::to(next).with(Effect::Preview(None));
                    }
                }
                State::Open { .. } => return Step::stay(),
            },
            Event::Opened => match state {
                State::Closed { next: opening } => next = State::open_with(opening.clone()),
                State::Open { .. } => return Step::stay(),
            },
            Event::Closing => match state {
                State::Closed { .. } => next = State::closed(),
                State::Open { .. } => return Step::stay(),
            },
            Event::Closed => {
                let previewed = matches!(
                    state,
                    State::Open {
                        previewed: Some(_),
                        ..
                    }
                );
                next = State::closed();
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
        }
        Step::to(next)
    }
}
