//! Role: Client callbacks, commit boundaries and thread-safe wake handles.

use super::write::Writer;
use kivact_wire::{HostEvent, HostInfo, Revision, TurnOutput};
use std::fmt;
use std::sync::Arc;

/// Thread-safe wake handle: calling it asks the host to run a turn soon.
/// Worker threads call it when a blocking job completes.
#[derive(Clone)]
pub struct HostWaker(Arc<dyn Fn() + Send + Sync>);

impl HostWaker {
    /// Build a wake handle from the host notification for pending client work.
    pub fn new(f: impl Fn() + Send + Sync + 'static) -> Self {
        HostWaker(Arc::new(f))
    }

    /// A no-op wake handle for drivers that explicitly poll pending work and
    /// need no external notification.
    pub fn noop() -> Self {
        HostWaker(Arc::new(|| {}))
    }

    /// Notify the host that pending client work should receive another turn.
    pub fn wake(&self) {
        (self.0)()
    }
}

impl fmt::Debug for HostWaker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HostWaker")
    }
}

/// An engine/client commit boundary among the events of a turn: the events
/// before index `events` include every event the writes of the revisions up
/// to `revision` produced. Every boundary of a turn is laid out: the client
/// runs a revision's `on_mount` callbacks after the events before its
/// boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// Number of events before this revision boundary in the delivered prefix.
    pub events: usize,
    /// Client revision whose writes are laid out before its mount callbacks.
    pub revision: Revision,
}

type StartCallback = dyn FnMut(&HostInfo, HostWaker, Writer);
type TurnCallback = dyn FnMut(&[HostEvent], &[Boundary], u64) -> TurnOutput;

/// Concrete callbacks for the engine's synchronous client boundary.
///
/// Two callbacks and no trait: `start` gets the [`HostInfo`], a
/// [`HostWaker`] and the [`Writer`] once; every later call is `turn`, with
/// the queued events, the [`Boundary`] of each revision laid out among them
/// (what the client mounts after which prefix: no separate post-layout
/// callback) and the host clock. Events after the last boundary (image
/// failures, layout reports of the layout pass) are delivered as ordinary
/// events of the turn.
pub struct Client {
    start: Box<StartCallback>,
    turn: Box<TurnCallback>,
}

impl Client {
    /// Supply the startup and the turn callbacks. A turn gets every queued
    /// event, the commit boundaries among them and the host clock.
    pub fn new(
        start: impl FnMut(&HostInfo, HostWaker, Writer) + 'static,
        turn: impl FnMut(&[HostEvent], &[Boundary], u64) -> TurnOutput + 'static,
    ) -> Self {
        Self {
            start: Box::new(start),
            turn: Box::new(turn),
        }
    }

    pub(crate) fn start(&mut self, info: &HostInfo, waker: HostWaker, writer: Writer) {
        (self.start)(info, waker, writer);
    }

    pub(crate) fn turn(
        &mut self,
        events: &[HostEvent],
        boundaries: &[Boundary],
        now_ns: u64,
    ) -> TurnOutput {
        (self.turn)(events, boundaries, now_ns)
    }
}
