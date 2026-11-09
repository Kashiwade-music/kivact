//! Role: Own combobox query, presentation and highlight transitions atomically.
use crate::surface::Surface;
use kivact_core::state::{Binding, IntoValue, Machine as MachineTrait, Step};
use std::marker::PhantomData;
#[derive(Clone)]
pub(super) enum Query {
    Local(String),
    Shared(Binding<String>),
}
impl PartialEq for Query {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Local(a), Self::Local(b)) => a == b,
            (Self::Shared(a), Self::Shared(b)) => a.into_value() == b.into_value(),
            _ => false,
        }
    }
}
impl Query {
    pub(super) fn get(&self) -> String {
        match self {
            Self::Local(text) => text.clone(),
            Self::Shared(source) => source.get(),
        }
    }
}
#[derive(Clone, PartialEq)]
pub(super) struct ComboboxState<K> {
    pub(super) mode: ComboMode,
    pub(super) query: Query,
    pub(super) highlight: Option<K>,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum ComboMode {
    Closed,
    Browsing(Surface),
    Filtering(Surface),
}
impl ComboMode {
    fn surface(self) -> Surface {
        match self {
            Self::Browsing(surface) | Self::Filtering(surface) => surface,
            Self::Closed => Surface::Popover,
        }
    }
    pub(super) fn is_open(self) -> bool {
        !matches!(self, Self::Closed)
    }
}
/// The close transition's reset of the query and the highlight.
fn reset<K: Clone>(state: &ComboboxState<K>, next: &mut ComboboxState<K>) -> Option<Effect> {
    next.highlight = None;
    match state.query {
        Query::Shared(source) => Some(Effect::Query(source, String::new())),
        Query::Local(_) => {
            next.query = Query::Local(String::new());
            None
        }
    }
}
pub(super) enum Event<K> {
    Open,
    OpenSheet,
    Close,
    /// The caller's open binding changed; the machine is its source of truth.
    OpenRequested(bool),
    /// The attached field lost the focus; `transferred` keeps the popup open
    /// for a presentation that owns the focus instead.
    FocusLeft {
        transferred: bool,
    },
    Share(Binding<String>),
    Query(String),
    QueryChanged,
    Highlight(K),
    /// Clear the query and the highlight without closing (a multiple choice
    /// clears the search and keeps the list open).
    Reset,
}
pub(super) enum Effect {
    Query(Binding<String>, String),
    WriteOpen(bool),
}
pub(super) struct Machine<K>(PhantomData<fn() -> K>);
impl<K> Machine<K> {
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}
impl<K: Clone + PartialEq + 'static> MachineTrait for Machine<K> {
    type State = ComboboxState<K>;
    type Event = Event<K>;
    type Effect = Effect;
    fn transition(
        &self,
        state: &ComboboxState<K>,
        event: Event<K>,
    ) -> Step<ComboboxState<K>, Event<K>, Effect> {
        let mut next = state.clone();
        match event {
            Event::Share(source) => {
                next.query = Query::Shared(source);
                next.highlight = None;
            }
            Event::QueryChanged => {
                next.mode = ComboMode::Filtering(state.mode.surface());
                next.highlight = None;
            }
            Event::Open => {
                next.mode = ComboMode::Browsing(Surface::Popover);
                return Step::to(next).with(Effect::WriteOpen(true));
            }
            Event::OpenSheet => {
                next.mode = ComboMode::Browsing(Surface::Sheet);
                return Step::to(next).with(Effect::WriteOpen(true));
            }
            Event::OpenRequested(true) if state.mode.is_open() => return Step::stay(),
            Event::OpenRequested(true) => {
                next.mode = ComboMode::Browsing(Surface::Popover);
                return Step::to(next).with(Effect::WriteOpen(true));
            }
            Event::OpenRequested(false) | Event::Close => {
                let write = state.mode.is_open();
                let reset = reset(state, &mut next);
                next.mode = ComboMode::Closed;
                let mut step = Step::to(next);
                if let Some(effect) = reset {
                    step = step.with(effect);
                }
                if write {
                    step = step.with(Effect::WriteOpen(false));
                }
                return step;
            }
            Event::FocusLeft { transferred } => {
                if transferred || !state.mode.is_open() {
                    return Step::stay();
                }
                let reset = reset(state, &mut next);
                next.mode = ComboMode::Closed;
                let mut step = Step::to(next);
                if let Some(effect) = reset {
                    step = step.with(effect);
                }
                return step.with(Effect::WriteOpen(false));
            }
            Event::Highlight(key) => next.highlight = Some(key),
            Event::Query(text) => {
                next.mode = ComboMode::Filtering(state.mode.surface());
                next.highlight = None;
                match state.query {
                    Query::Shared(source) => {
                        return Step::to(next).with(Effect::Query(source, text));
                    }
                    Query::Local(_) => next.query = Query::Local(text),
                }
            }
            Event::Reset => {
                if let Some(effect) = reset(state, &mut next) {
                    return Step::to(next).with(effect);
                }
            }
        }
        Step::to(next)
    }
}
