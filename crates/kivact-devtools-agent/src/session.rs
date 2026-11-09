//! Role: Who is connected, what they follow, and what every connection thread reads.

use std::collections::BTreeMap;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};

use kivact_devtools_protocol::{AppInfo, ClientInfo, ClientKind, DevtoolsConfig, Topic};

pub(crate) type ConnectionId = u64;
/// The write half of a connection; a writer holds the lock for one whole line.
pub(crate) type Writer = Arc<Mutex<TcpStream>>;

/// What a connection thread knows about its own connection.
pub(crate) enum Session {
    AwaitingInitialize,
    Ready,
}

/// An initialized connection.
pub(crate) struct Peer {
    pub(crate) client: ClientInfo,
    pub(crate) topics: Vec<Topic>,
    pub(crate) writer: Writer,
}

/// Every initialized connection.
#[derive(Default)]
pub(crate) struct Peers(Mutex<BTreeMap<ConnectionId, Peer>>);

impl Peers {
    /// The map of peers; hold the lock for one short step only.
    fn lock(&self) -> MutexGuard<'_, BTreeMap<ConnectionId, Peer>> {
        self.0
            .lock()
            .expect("invariant: the peers lock is never poisoned")
    }

    pub(crate) fn insert(&self, id: ConnectionId, peer: Peer) {
        self.lock().insert(id, peer);
    }

    /// Remove one peer and return how many are still connected.
    pub(crate) fn remove(&self, id: ConnectionId) -> usize {
        let mut peers = self.lock();
        peers.remove(&id);
        peers.len()
    }

    pub(crate) fn set_topics(&self, id: ConnectionId, topics: Vec<Topic>) {
        if let Some(peer) = self.lock().get_mut(&id) {
            peer.topics = topics;
        }
    }

    pub(crate) fn wants(&self, topic: Topic) -> bool {
        self.lock()
            .values()
            .any(|peer| peer.topics.contains(&topic))
    }

    pub(crate) fn writers_for(&self, topic: Topic) -> Vec<Writer> {
        self.lock()
            .values()
            .filter(|peer| peer.topics.contains(&topic))
            .map(|peer| Arc::clone(&peer.writer))
            .collect()
    }

    pub(crate) fn has_client(&self, kind: ClientKind) -> bool {
        self.lock().values().any(|peer| peer.client.kind == kind)
    }

    /// Shut every connection down, so its reader thread ends and cleans up.
    pub(crate) fn shutdown_all(&self) {
        let writers: Vec<Writer> = self
            .lock()
            .values()
            .map(|peer| Arc::clone(&peer.writer))
            .collect();
        for writer in writers {
            shutdown(&writer);
        }
    }
}

/// What every connection thread reads and nobody changes after start.
pub(crate) struct Shared {
    pub(crate) token: String,
    pub(crate) app: AppInfo,
    pub(crate) config: DevtoolsConfig,
    pub(crate) peers: Peers,
    pub(crate) wake: Box<dyn Fn() + Send + Sync>,
}

/// Write one line and `\n`. A failed write shuts the stream down, so its reader thread ends and cleans up.
pub(crate) fn write_line(writer: &Writer, line: &str) {
    let mut stream = writer
        .lock()
        .expect("invariant: the writer lock is never poisoned");
    let written = stream
        .write_all(line.as_bytes())
        .and_then(|()| stream.write_all(b"\n"))
        .and_then(|()| stream.flush());
    if written.is_err() {
        let _ = stream.shutdown(Shutdown::Both);
    }
}

pub(crate) fn shutdown(writer: &Writer) {
    let stream = writer
        .lock()
        .expect("invariant: the writer lock is never poisoned");
    let _ = stream.shutdown(Shutdown::Both);
}
