//! Role: Own the combobox open state, filter phase and highlight in one step.
use crate::surface::Surface;
use kivact_core::state::{Machine as MachineTrait, Step};
use std::marker::PhantomData;

/// Whether the list shows, and while it shows, its surface, whether the
/// query was typed since it opened, and the highlighted option.
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
    pub(super) fn is_open(&self) -> bool {
        matches!(self, Self::Open { .. })
    }
    pub(super) fn is_filtering(&self) -> bool {
        matches!(
            self,
            Self::Open {
                filtering: true,
                ..
            }
        )
    }
    pub(super) fn surface(&self) -> Option<Surface> {
        match self {
            Self::Open { surface, .. } => Some(*surface),
            Self::Closed => None,
        }
    }
    pub(super) fn highlight(&self) -> Option<&K> {
        match self {
            Self::Open { highlight, .. } => highlight.as_ref(),
            Self::Closed => None,
        }
    }
}
pub(super) enum Event<K> {
    /// Show the list in the popover.
    Open,
    /// Show the list in the touch sheet; an open popover moves to the sheet.
    OpenSheet,
    Close,
    /// The caller's open binding, or the palette's open, changed.
    OpenRequested(bool),
    /// The attached field lost the focus; `transferred` keeps the list
    /// open for a presentation that owns the focus instead.
    FocusLeft {
        transferred: bool,
    },
    /// The user typed this query.
    Query(String),
    /// The caller wrote the shared query.
    QueryChanged,
    Highlight(K),
    /// A multiple choice toggled an option: the search clears and the
    /// list stays open.
    Toggled,
}
pub(super) enum Effect {
    /// Write this text to the query binding.
    WriteQuery(String),
    /// Write the caller's open binding.
    WriteOpen(bool),
}
/// The transition, with the presentation fixed at setup.
pub(super) struct Machine<K> {
    /// An inline list (the command palette's) closes only when its owner
    /// closes it, and its next opening starts from an empty query. A popup
    /// keeps its query when it closes: the field still shows the text.
    inline: bool,
    key: PhantomData<fn() -> K>,
}
impl<K> Machine<K> {
    pub(super) fn new(inline: bool) -> Self {
        Self {
            inline,
            key: PhantomData,
        }
    }
    fn open(surface: Surface, highlight: Option<K>) -> Step<ComboState<K>, Event<K>, Effect> {
        Step::to(ComboState::Open {
            surface,
            filtering: false,
            highlight,
        })
        .with(Effect::WriteOpen(true))
    }
    /// Close an open list. The highlight goes with the open state; an
    /// inline list also clears its query.
    fn close(&self, state: &ComboState<K>) -> Step<ComboState<K>, Event<K>, Effect> {
        if !state.is_open() {
            return Step::stay();
        }
        let step = Step::to(ComboState::Closed).with(Effect::WriteOpen(false));
        if self.inline {
            step.with(Effect::WriteQuery(String::new()))
        } else {
            step
        }
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
        match event {
            Event::Open | Event::OpenRequested(true) if state.is_open() => Step::stay(),
            Event::Open | Event::OpenRequested(true) => Self::open(Surface::Popover, None),
            Event::OpenSheet => Self::open(Surface::Sheet, state.highlight().cloned()),
            Event::Close | Event::OpenRequested(false) => self.close(state),
            Event::FocusLeft { transferred: true } => Step::stay(),
            Event::FocusLeft { transferred: false } => self.close(state),
            Event::Query(text) => {
                let step = Step::to(ComboState::Open {
                    surface: state.surface().unwrap_or(Surface::Popover),
                    filtering: true,
                    highlight: None,
                })
                .with(Effect::WriteQuery(text));
                if state.is_open() {
                    step
                } else {
                    step.with(Effect::WriteOpen(true))
                }
            }
            Event::QueryChanged => match state {
                ComboState::Open { surface, .. } => Step::to(ComboState::Open {
                    surface: *surface,
                    filtering: true,
                    highlight: None,
                }),
                ComboState::Closed => Step::stay(),
            },
            Event::Highlight(key) => match state {
                ComboState::Open {
                    surface, filtering, ..
                } => Step::to(ComboState::Open {
                    surface: *surface,
                    filtering: *filtering,
                    highlight: Some(key),
                }),
                ComboState::Closed => Step::stay(),
            },
            Event::Toggled => match state {
                ComboState::Open {
                    surface, filtering, ..
                } => Step::to(ComboState::Open {
                    surface: *surface,
                    filtering: *filtering,
                    highlight: None,
                })
                .with(Effect::WriteQuery(String::new())),
                ComboState::Closed => Step::stay().with(Effect::WriteQuery(String::new())),
            },
        }
    }
}
