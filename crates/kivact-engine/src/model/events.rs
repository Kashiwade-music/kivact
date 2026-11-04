//! Role: Orders host events and private post-layout commit boundaries.

use crate::contract::HostEvent;
use kivact_types::Revision;
use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub(crate) struct EventQueue {
    events: VecDeque<HostEvent>,
    boundaries: VecDeque<(usize, Revision)>,
    revision: Revision,
}

impl EventQueue {
    pub(crate) fn push(&mut self, event: HostEvent) {
        self.events.push_back(event);
    }

    pub(crate) fn has_events(&self) -> bool {
        !self.events.is_empty() || !self.boundaries.is_empty()
    }

    /// Take every queued event in order, with the commit boundaries among them: the
    /// number of events before each boundary and its revision.
    pub(crate) fn drain(&mut self) -> (Vec<HostEvent>, Vec<(usize, Revision)>) {
        (
            self.events.drain(..).collect(),
            self.boundaries.drain(..).collect(),
        )
    }

    pub(crate) fn applied(&self) -> Revision {
        self.revision
    }

    pub(crate) fn commit(&mut self, revision: Revision) {
        self.revision = revision;
        self.boundaries.push_back((self.events.len(), revision));
    }
}
