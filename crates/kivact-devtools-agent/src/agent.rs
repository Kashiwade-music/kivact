//! Role: The agent a target app runs: start, incoming calls, notifications and its files.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};

use kivact_devtools_protocol::{
    AppInfo, ClientKind, DevtoolsConfig, DiscoveryFile, Empty, Notification, PROTOCOL_VERSION,
    SourceLocation, Topic, notification_line, rfc3339_utc,
};

use crate::config;
use crate::discovery;
use crate::files;
use crate::incoming::Incoming;
use crate::launcher;
use crate::review_store::ReviewStore;
use crate::server;
use crate::session::{Peers, Shared, write_line};
use crate::source::SourceCache;

/// What the core passes to `Agent::start`. `app.workspace_root` is passed empty; the agent finds
/// the workspace root, reads `.kivact/devtools.json` and fills it in.
pub struct AgentConfig {
    /// The app as the core knows it.
    pub app: AppInfo,
}

/// The DevTools agent of one target app.
pub struct Agent {
    shared: Arc<Shared>,
    incoming: mpsc::Receiver<Incoming>,
    stop: mpsc::Sender<()>,
    listener: Option<std::thread::JoinHandle<()>>,
    port: u16,
    discovery_path: PathBuf,
    workspace_root: PathBuf,
    review: ReviewStore,
    names: SourceCache,
    session: String,
}

impl Agent {
    /// Bind, write the discovery file and start the listener thread. `wake` is called from other
    /// threads when something arrives.
    pub fn start(config: AgentConfig, wake: Box<dyn Fn() + Send + Sync>) -> std::io::Result<Agent> {
        let workspace_root = config::find_workspace_root();
        let devtools_config = config::load_config(&workspace_root);
        let mut app = config.app;
        app.workspace_root = workspace_root.to_string_lossy().into_owned();
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let token = make_token();
        let started_ms = now_ms();
        let session = format!("{}-{}-{}", compact_utc(started_ms), app.pid, port);
        let discovery_path = discovery::write(&DiscoveryFile {
            protocol: PROTOCOL_VERSION,
            pid: app.pid,
            app_name: app.name.clone(),
            kivact_version: app.kivact_version.clone(),
            port: u32::from(port),
            token: token.clone(),
            started_unix_ms: started_ms,
            workspace_root: app.workspace_root.clone(),
        })?;
        let shared = Arc::new(Shared {
            token,
            app,
            config: devtools_config,
            peers: Peers::default(),
            wake,
        });
        let (incoming_tx, incoming) = mpsc::channel();
        let (stop, stop_rx) = mpsc::channel();
        let shared_for_thread = Arc::clone(&shared);
        let listener_thread = std::thread::Builder::new()
            .name("kivact-devtools-listener".to_owned())
            .spawn(move || server::listen(listener, stop_rx, shared_for_thread, incoming_tx))?;
        let review = ReviewStore::new(
            workspace_root.join(".kivact").join("review").join(&session),
            shared.app.clone(),
            session.clone(),
            started_ms,
        );
        Ok(Agent {
            shared,
            incoming,
            stop,
            listener: Some(listener_thread),
            port,
            discovery_path,
            workspace_root,
            review,
            names: SourceCache::new(),
            session,
        })
    }

    /// Everything that arrived since the last time, in order, on the UI thread. The caller runs
    /// the calls after releasing its borrow of the agent.
    pub fn take_incoming(&mut self) -> Vec<Incoming> {
        self.incoming.try_iter().collect()
    }

    /// Send a notification to the connections subscribed to its topic.
    pub fn notify(&self, notification: Notification) {
        let writers = self.shared.peers.writers_for(notification.topic());
        if writers.is_empty() {
            return;
        }
        let line = notification_line(&notification);
        for writer in writers {
            write_line(&writer, &line);
        }
    }

    /// Whether any connection subscribed to `topic`.
    pub fn wants(&self, topic: Topic) -> bool {
        self.shared.peers.wants(topic)
    }

    /// Notify the connected DevTools app (`focus_requested`), or start one.
    pub fn open_devtools(&mut self) -> Result<(), String> {
        if self.shared.peers.has_client(ClientKind::Devtools) {
            self.notify(Notification::FocusRequested(Empty {}));
            return Ok(());
        }
        launcher::launch(
            &self.shared.config,
            &self.workspace_root,
            &self.discovery_path,
        )
    }

    /// The configuration as read (no defaults filled in).
    pub fn config(&self) -> &DevtoolsConfig {
        &self.shared.config
    }

    pub fn discovery_path(&self) -> &Path {
        &self.discovery_path
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// The app as announced to clients (`workspace_root` filled in): the answer of `get_app_info`.
    pub fn app(&self) -> &AppInfo {
        &self.shared.app
    }

    /// The review of this session (owned here; saved on every change).
    pub fn review(&mut self) -> &mut ReviewStore {
        &mut self.review
    }

    /// The identifier after `let` on the line at `location`, if any.
    pub fn name_hint(&mut self, location: &SourceLocation) -> Option<String> {
        self.names.name_hint(&self.workspace_root, location)
    }

    /// Write a trace file and return its path.
    pub fn write_trace(&self, contents: &str, path: Option<&str>) -> std::io::Result<PathBuf> {
        let path = files::trace_path(&self.workspace_root, &self.session, path);
        files::write_atomic(&path, contents.as_bytes())?;
        Ok(path)
    }
}

impl Drop for Agent {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        let _ = std::net::TcpStream::connect(("127.0.0.1", self.port));
        if let Some(listener) = self.listener.take() {
            let _ = listener.join();
        }
        self.shared.peers.shutdown_all();
        discovery::remove(&self.discovery_path);
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

/// `20261108-053200` from the Unix time (UTC).
fn compact_utc(unix_ms: u64) -> String {
    rfc3339_utc(unix_ms)
        .replace(['-', ':'], "")
        .replace('T', "-")
        .trim_end_matches('Z')
        .to_owned()
}

/// 32 random bytes as hex. No external crate: `RandomState` is seeded by the OS. Loopback only, so
/// this only has to be hard to guess, not cryptographic.
fn make_token() -> String {
    use std::hash::{BuildHasher, Hasher};

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    (0..4_u8)
        .map(|round| {
            let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
            hasher.write_u128(nanos);
            hasher.write_u32(std::process::id());
            hasher.write_u8(round);
            format!("{:016x}", hasher.finish())
        })
        .collect()
}
