//! Role: Own the combobox list's per-opening presentation, filter phase and highlight.
use crate::surface::Surface;
use kivact_core::state::{Machine as MachineTrait, Step};
use std::marker::PhantomData;

/// How the current opening presents the list; the caller's `open` says whether it shows.
#[derive(Clone, PartialEq)]
pub(super) enum ComboState<K> {
    Closed,
    Open {
        surface: Surface,
        /// The query was typed, or written by the caller, while open.
        filtering: bool,
        highlight: Option<K>,
    },
}

impl<K> ComboState<K> {
    pub(super) fn surface(&self) -> Option<Surface> {
        match self {
            Self::Open { surface, .. } => Some(*surface),
            Self::Closed => None,
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
            Self::Closed => None,
        }
    }
}

pub(super) enum Event<K> {
    /// The caller's `open` became true.
    Opened,
    /// Show the list in this surface; an open list moves to it.
    Present(Surface),
    /// The caller's `open` became false.
    Closed,
    /// The user typed into the field.
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
            (Closed, Event::Opened) => Step::to(Open {
                surface: Surface::Popover,
                filtering: false,
                highlight: None,
            }),
            (Closed, Event::Present(surface)) => Step::to(Open {
                surface,
                filtering: false,
                highlight: None,
            }),
            (Open { highlight, .. }, Event::Present(surface)) => Step::to(Open {
                surface,
                filtering: false,
                highlight: highlight.clone(),
            }),
            (Open { .. }, Event::Closed) => Step::to(Closed).with(Effect::Closed),
            (Closed, Event::Typed) => Step::to(Open {
                surface: Surface::Popover,
                filtering: true,
                highlight: None,
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
