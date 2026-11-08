//! Role: Orders host events and private post-layout commit boundaries.
//!
//! The event queue is shared by every window's model (`EventSink`): the
//! App has one event stream, so a turn sees the events of every window in
//! the order they were queued.

use crate::contract::HostEvent;
use crate::lower::assets::ArvaneAssets;
use kivact_types::Revision;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
struct EventQueue {
    events: VecDeque<HostEvent>,
    boundaries: VecDeque<(usize, Revision)>,
    revision: Revision,
}

impl EventQueue {
    fn push(&mut self, event: HostEvent) {
        self.events.push_back(event);
    }

    fn has_events(&self) -> bool {
        !self.events.is_empty() || !self.boundaries.is_empty()
    }

    /// Take every queued event in order, with the commit boundaries among them: the
    /// number of events before each boundary and its revision.
    fn drain(&mut self) -> (Vec<HostEvent>, Vec<(usize, Revision)>) {
        (
            self.events.drain(..).collect(),
            self.boundaries.drain(..).collect(),
        )
    }

    fn applied(&self) -> Revision {
        self.revision
    }

    fn commit(&mut self, revision: Revision) {
        self.revision = revision;
        self.boundaries.push_back((self.events.len(), revision));
    }
}

/// The App's one event queue, shared by every window's model.
#[derive(Clone, Default)]
pub(crate) struct EventSink(Rc<RefCell<EventQueue>>);

impl EventSink {
    pub(crate) fn push(&self, event: HostEvent) {
        self.0.borrow_mut().push(event);
    }

    pub(crate) fn has_events(&self) -> bool {
        self.0.borrow().has_events()
    }

    pub(crate) fn drain(&self) -> (Vec<HostEvent>, Vec<(usize, Revision)>) {
        self.0.borrow_mut().drain()
    }

    pub(crate) fn applied(&self) -> Revision {
        self.0.borrow().applied()
    }

    pub(crate) fn commit(&self, revision: Revision) {
        self.0.borrow_mut().commit(revision);
    }
}

/// What every window's model shares: the event queue and the asset store.
pub(crate) struct Shared {
    pub(crate) events: EventSink,
    pub(crate) assets: Rc<RefCell<ArvaneAssets>>,
}
