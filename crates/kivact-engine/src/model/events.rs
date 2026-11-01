//! Role: Orders host events and private post-layout commit boundaries.

use kivact_wire::{HostEvent, Revision};
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

    pub(crate) fn take(&mut self) -> Vec<HostEvent> {
        debug_assert!(self.boundaries.is_empty());
        self.events.drain(..).collect()
    }

    /// Drain only events preceding the oldest completed commit.
    pub(crate) fn take_commit(&mut self) -> Option<(Vec<HostEvent>, Revision)> {
        let (offset, revision) = self.boundaries.pop_front()?;
        let events = self.events.drain(..offset).collect();
        for (following, _) in &mut self.boundaries {
            *following -= offset;
        }
        Some((events, revision))
    }

    pub(crate) fn applied(&self) -> Revision {
        self.revision
    }

    pub(crate) fn commit(&mut self, revision: Revision) {
        self.revision = revision;
        self.boundaries.push_back((self.events.len(), revision));
    }
}
