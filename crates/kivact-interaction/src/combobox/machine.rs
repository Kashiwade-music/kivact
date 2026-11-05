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
}
pub(super) enum Event<K> {
    Open,
    OpenSheet,
    Close,
    Share(Binding<String>),
    Query(String),
    QueryChanged,
    Highlight(K),
    Reset,
}
pub(super) enum Effect {
    Query(Binding<String>, String),
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
                next.highlight = None;
            }
            Event::Open => next.mode = ComboMode::Browsing(Surface::Popover),
            Event::OpenSheet => next.mode = ComboMode::Browsing(Surface::Sheet),
            Event::Close => next.mode = ComboMode::Closed,
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
                next.highlight = None;
                match state.query {
                    Query::Shared(source) => {
                        return Step::to(next).with(Effect::Query(source, String::new()));
                    }
                    Query::Local(_) => next.query = Query::Local(String::new()),
                }
            }
        }
        Step::to(next)
    }
}
