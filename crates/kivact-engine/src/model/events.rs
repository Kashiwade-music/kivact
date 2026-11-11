//! Role: Orders host events and private post-layout commit boundaries.
//!
//! The event queue is shared by every window's model (`EventSink`): the
//! App has one event stream, so a turn sees the events of every window in
//! the order they were queued.

use crate::contract::{EngineRecords, FrameRecord, HostEvent, SyncRecord};
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
