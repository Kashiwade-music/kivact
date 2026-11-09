//! Role: Own the notification queue, visibility limit and independent hold identities.
use super::data::{DismissHandler, FocusAttachment, QueuedToast, ToastId};
use kivact_core::custom::Generation;
use kivact_core::custom::NodeToken;
use kivact_core::state::{Machine as MachineTrait, Step};
use kivact_core::view::DismissReason;
use std::collections::BTreeSet;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct HoldId(pub Generation);
#[derive(Clone, Default, PartialEq)]
pub(super) struct HoldSet(BTreeSet<HoldId>);
impl HoldSet {
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
#[derive(Clone, PartialEq)]
pub(super) struct State {
    pub queue: Vec<QueuedToast>,
    pub limit: usize,
    pub holds: HoldSet,
}
impl Default for State {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            limit: 3,
            holds: HoldSet::default(),
        }
    }
}
pub(super) enum Event {
    Push(QueuedToast),
    Remove(ToastId, DismissReason, Option<NodeToken>),
    Limit(usize),
    Hold(HoldId, bool),
    Focus(ToastId, FocusAttachment),
}
pub(super) enum Effect {
    Dismiss {
        // xtask-allow(state-fields): passive record - the focus target to restore, absent when there is none
        previous: Option<NodeToken>,
        // xtask-allow(state-fields): passive record - the caller's dismiss callback, absent when there is none
        callback: Option<DismissHandler>,
        reason: DismissReason,
    },
}
pub(super) struct Machine;
impl MachineTrait for Machine {
    type State = State;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &State, event: Event) -> Step<State, Event, Effect> {
        let mut state = state.clone();
        match event {
            Event::Focus(id, focus) => {
                if let Some(toast) = state.queue.iter_mut().find(|toast| toast.id == id) {
                    toast.focus = focus;
                }
            }
            Event::Push(toast) => state.queue.push(toast),
            Event::Remove(id, reason, previous) => {
                let Some(index) = state.queue.iter().position(|toast| toast.id == id) else {
                    return Step::stay();
                };
                let toast = state.queue.remove(index);
                let callback = (index < state.limit)
                    .then_some(toast.options.on_dismiss)
                    .flatten();
                return Step::to(state).with(Effect::Dismiss {
                    previous,
                    callback,
                    reason,
                });
            }
            Event::Limit(limit) => state.limit = limit,
            Event::Hold(id, true) => {
                state.holds.0.insert(id);
            }
            Event::Hold(id, false) => {
                state.holds.0.remove(&id);
            }
        }
        Step::to(state)
    }
}
