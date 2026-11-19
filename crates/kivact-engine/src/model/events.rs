//! Role: Orders host events and private post-layout commit boundaries.
//!
//! The event queue is shared by every window's model (`EventSink`): the
//! App has one event stream, so a turn sees the events of every window in
//! the order they were queued.

use crate::contract::{EngineRecords, FrameRecord, HostEvent, SyncRecord};
use crate::lower::assets::ArvaneAssets;
use kivact_types::{NodeId, Revision, WindowId};
use std::cell::RefCell;
use std::collections::{BTreeSet, VecDeque};
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
    /// number of events before each boundary and its revision. The queue
    /// hands its buffers over, so it keeps no capacity of a burst.
    fn drain(&mut self) -> (Vec<HostEvent>, Vec<(usize, Revision)>) {
        let events: Vec<HostEvent> = std::mem::take(&mut self.events).into();
        let boundaries: Vec<(usize, Revision)> = std::mem::take(&mut self.boundaries).into();
        (events, boundaries)
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

/// What the DevTools records: one log shared by every window's Model and the Host.
#[derive(Clone, Default)]
pub(crate) struct EngineLog(Rc<RefCell<EngineLogState>>);

/// The two record rings of the one log.
#[derive(Default)]
pub(crate) struct EngineLogState {
    pub(crate) syncs: Log<SyncRecord>,
    pub(crate) frames: Log<FrameRecord>,
    pub(crate) tree: TreeJournal,
}

/// The tree changes the DevTools records (see `contract::TreeChanges`).
#[derive(Default)]
pub(crate) enum TreeJournal {
    #[default]
    Off,
    Recording {
        upserted: BTreeSet<(WindowId, NodeId)>,
        removed: BTreeSet<(WindowId, NodeId)>,
    },
}

/// A log of recent records: the ring exists only while recording.
#[derive(Default)]
pub(crate) enum Log<T> {
    #[default]
    Off,
    Recording {
        ring: VecDeque<T>,
        dropped: u32,
    },
}

impl<T> Log<T> {
    /// Turn the ring on (empty) or off (discarding its records).
    fn set(&mut self, on: bool) {
        match (on, &*self) {
            (true, Self::Off) => {
                *self = Self::Recording {
                    ring: VecDeque::new(),
                    dropped: 0,
                };
            }
            (false, Self::Recording { .. }) => *self = Self::Off,
            _ => {}
        }
    }

    /// Keep `make`'s record in the ring, dropping the oldest beyond `cap`.
    fn push(&mut self, cap: usize, make: impl FnOnce() -> T) {
        let Self::Recording { ring, dropped } = self else {
            return;
        };
        if ring.len() >= cap {
            ring.pop_front();
            *dropped = dropped.saturating_add(1);
        }
        ring.push_back(make());
    }

    /// Take the records and the number dropped, keeping the ring recording.
    fn drain(&mut self) -> (Vec<T>, u32) {
        match self {
            Self::Off => (Vec::new(), 0),
            Self::Recording { ring, dropped } => {
                (ring.drain(..).collect(), std::mem::take(dropped))
            }
        }
    }
}

impl EngineLog {
    /// Turn the sync and frame records on or off.
    pub(crate) fn set(&self, syncs: bool, frames: bool) {
        let mut state = self.0.borrow_mut();
        state.syncs.set(syncs);
        state.frames.set(frames);
    }

    /// Whether frames are being recorded.
    pub(crate) fn frames_recording(&self) -> bool {
        matches!(self.0.borrow().frames, Log::Recording { .. })
    }

    /// Whether syncs are being recorded.
    pub(crate) fn syncs_recording(&self) -> bool {
        matches!(self.0.borrow().syncs, Log::Recording { .. })
    }

    /// Keep one sync record (at most 4096).
    pub(crate) fn sync(&self, make: impl FnOnce() -> SyncRecord) {
        self.0.borrow_mut().syncs.push(4096, make);
    }

    /// Keep one frame record (at most 512).
    pub(crate) fn frame(&self, make: impl FnOnce() -> FrameRecord) {
        self.0.borrow_mut().frames.push(512, make);
    }

    /// Take both rings.
    pub(crate) fn take(&self) -> EngineRecords {
        let mut state = self.0.borrow_mut();
        let (syncs, sync_dropped) = state.syncs.drain();
        let (frames, frame_dropped) = state.frames.drain();
        EngineRecords {
            syncs,
            frames,
            dropped: sync_dropped.saturating_add(frame_dropped),
        }
    }

    /// Start (keeping what is recorded) or stop recording the tree changes.
    pub(crate) fn record_tree(&self, on: bool) {
        let mut state = self.0.borrow_mut();
        let recording = matches!(state.tree, TreeJournal::Recording { .. });
        if on && !recording {
            state.tree = TreeJournal::Recording {
                upserted: BTreeSet::new(),
                removed: BTreeSet::new(),
            };
        } else if !on {
            state.tree = TreeJournal::Off;
        }
    }

    /// The client created or wrote these nodes of `window` (nothing unless recording).
    pub(crate) fn tree_upserted(&self, window: WindowId, ids: impl IntoIterator<Item = NodeId>) {
        if let TreeJournal::Recording { upserted, .. } = &mut self.0.borrow_mut().tree {
            upserted.extend(ids.into_iter().map(|id| (window, id)));
        }
    }

    /// `id` of `window` was destroyed (nothing unless recording).
    pub(crate) fn tree_removed(&self, window: WindowId, id: NodeId) {
        if let TreeJournal::Recording { upserted, removed } = &mut self.0.borrow_mut().tree {
            upserted.remove(&(window, id));
            removed.insert((window, id));
        }
    }

    /// The tree changes recorded since the last call (empty when not recording).
    pub(crate) fn take_tree(&self) -> crate::contract::TreeChanges {
        match &mut self.0.borrow_mut().tree {
            TreeJournal::Off => crate::contract::TreeChanges::default(),
            TreeJournal::Recording { upserted, removed } => crate::contract::TreeChanges {
                upserted: std::mem::take(upserted).into_iter().collect(),
                removed: std::mem::take(removed).into_iter().collect(),
            },
        }
    }
}

/// What every window's model shares: the event queue, the asset store and
/// the DevTools' record log.
pub(crate) struct Shared {
    pub(crate) events: EventSink,
    pub(crate) assets: Rc<RefCell<ArvaneAssets>>,
    pub(crate) log: EngineLog,
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the queue's drain: the burst buffer is handed over in push order.

    use super::*;

    #[test]
    fn drain_hands_over_the_burst_buffer_in_push_order() {
        let mut queue = EventQueue::default();
        for turns in 0..5000 {
            queue.push(HostEvent::TurnsCut { turns });
        }
        let (events, boundaries) = queue.drain();
        assert_eq!(queue.events.capacity(), 0);
        assert!(boundaries.is_empty());
        assert_eq!(events.len(), 5000);
        assert!(
            events
                .iter()
                .enumerate()
                .all(|(i, event)| *event == HostEvent::TurnsCut { turns: i as u32 })
        );
    }
}
