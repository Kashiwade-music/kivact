//! Role: Atomic query and highlight transitions for comboboxes.
use kivact_core::state::{Machine as MachineTrait, Step};
use std::marker::PhantomData;

#[derive(Clone, PartialEq)]
pub(super) struct State<K> {
    pub(super) query: String,
    pub(super) highlight: Option<K>,
}
pub(super) enum Event<K> {
    Query(String),
    Highlight(K),
    Reset,
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
        match event {
            Event::Query(query) => Step::to(State {
                query,
                highlight: None,
            }),
            Event::Highlight(key) => Step::to(State {
                query: state.query.clone(),
                highlight: Some(key),
            }),
            Event::Reset => Step::to(State {
                query: String::new(),
                highlight: None,
            }),
        }
    }
}
