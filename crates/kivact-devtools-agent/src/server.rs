//! Role: The listener thread: accepts connections and starts a reader thread for each.

use std::net::TcpListener;
use std::sync::{Arc, mpsc};

use crate::connection;
use crate::incoming::Incoming;
use crate::session::{ConnectionId, Shared};

/// Accept until `stop` has a value or is disconnected. `Agent::drop` sends `()` and then
/// connects once, so `accept` returns.
pub(crate) fn listen(
    listener: TcpListener,
    stop: mpsc::Receiver<()>,
    shared: Arc<Shared>,
    incoming: mpsc::Sender<Incoming>,
) {
    let mut next_id: ConnectionId = 0;
    for stream in listener.incoming() {
        if !matches!(stop.try_recv(), Err(mpsc::TryRecvError::Empty)) {
            break;
        }
        let Ok(stream) = stream else {
            continue;
        };
        next_id += 1;
        let id = next_id;
        let (shared, incoming) = (Arc::clone(&shared), incoming.clone());
        let spawned = std::thread::Builder::new()
            .name(format!("kivact-devtools-connection-{id}"))
            .spawn(move || connection::run(stream, id, shared, incoming));
        if let Err(error) = spawned {
            tracing::warn!(
                target: "kivact::devtools",
                "cannot start a DevTools connection thread: {error}"
            );
        }
    }
}
