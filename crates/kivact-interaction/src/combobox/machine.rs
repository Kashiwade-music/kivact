//! Role: Own the combobox list's per-opening presentation, filter phase and highlight.
use crate::surface::Surface;
use kivact_core::state::{Machine as MachineTrait, Step};
use std::marker::PhantomData;

/// The details of the next opening, noted before the caller's `open` is written.
#[derive(Clone, PartialEq)]
pub(super) struct Opening {
    pub(super) surface: Surface,
    pub(super) filtering: bool,
}

impl Default for Opening {
    fn default() -> Self {
        Self {
            surface: Surface::Popover,
            filtering: false,
        }
    }
}

/// How the current opening presents the list; the caller's `open` says whether it shows.
#[derive(Clone, PartialEq)]
pub(super) enum ComboState<K> {
    /// Closed, holding the details the next opening takes.
    Closed { next: Opening },
    Open {
        surface: Surface,
        /// The query was typed, or written by the caller, while open.
        filtering: bool,
        highlight: Option<K>,
    },
}

impl<K> ComboState<K> {
    /// The state of a closed list, with the default next opening.
    pub(super) fn closed() -> Self {
        Self::Closed {
            next: Opening::default(),
        }
    }

    /// The state of a list opened with `opening`, nothing highlighted.
    pub(super) fn opened(opening: Opening) -> Self {
        Self::Open {
            surface: opening.surface,
            filtering: opening.filtering,
            highlight: None,
        }
    }

    pub(super) fn surface(&self) -> Option<Surface> {
        match self {
            Self::Open { surface, .. } => Some(*surface),
            Self::Closed { .. } => None,
        }
    }

    pub(super) fn filtering(&self) -> bool {
        matches!(
            self,
            Self::Open {
                filtering: true,
                ..
            }
        )
    }

    pub(super) fn highlight(&self) -> Option<&K> {
        match self {
            Self::Open { highlight, .. } => highlight.as_ref(),
            Self::Closed { .. } => None,
        }
    }
}

pub(super) enum Event<K> {
    /// The caller's `open` became true: the follow of `open`.
    Opened,
    /// Note the surface of the next opening; an open list moves to it.
    Present(Surface),
    /// The caller's `open` became false: the follow of `open`.
    Closed,
    /// The list is being hidden: a note noted before the hide must not outlive it.
    Closing,
    /// Note that the user typed: an open list filters, a closed one opens filtered.
    Typed,
    /// The query changed while the list is open.
    QueryChanged,
    Highlight(K),
}

pub(super) enum Effect {
    /// The list closed.
    Closed,
}

pub(super) struct Machine<K>(PhantomData<fn() -> K>);

impl<K> Machine<K> {
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<K: Clone + PartialEq + 'static> MachineTrait for Machine<K> {
    type State = ComboState<K>;
    type Event = Event<K>;
    type Effect = Effect;

    fn transition(
        &self,
        state: &ComboState<K>,
        event: Event<K>,
    ) -> Step<ComboState<K>, Event<K>, Effect> {
        use ComboState::{Closed, Open};
        match (state, event) {
            (Closed { next }, Event::Opened) => Step::to(Open {
                surface: next.surface,
                filtering: next.filtering,
                highlight: None,
            }),
            (Open { .. }, Event::Opened) => Step::stay(),
            (Closed { .. }, Event::Present(surface)) => Step::to(Closed {
                next: Opening {
                    surface,
                    filtering: false,
                },
            }),
            (Open { highlight, .. }, Event::Present(surface)) => Step::to(Open {
                surface,
                filtering: false,
                highlight: highlight.clone(),
            }),
            (Open { .. }, Event::Closed) => Step::to(ComboState::closed()).with(Effect::Closed),
            (Closed { .. }, Event::Closed) => Step::stay(),
            (Closed { .. }, Event::Closing) => Step::to(ComboState::closed()),
            (Closed { .. }, Event::Typed) => Step::to(Closed {
                next: Opening {
                    surface: Surface::Popover,
                    filtering: true,
                },
            }),
            (Open { surface, .. }, Event::Typed | Event::QueryChanged) => Step::to(Open {
                surface: *surface,
                filtering: true,
                highlight: None,
            }),
            (
                Open {
                    surface, filtering, ..
                },
                Event::Highlight(key),
            ) => Step::to(Open {
                surface: *surface,
                filtering: *filtering,
                highlight: Some(key),
            }),
            _ => Step::stay(),
        }
    }
}
