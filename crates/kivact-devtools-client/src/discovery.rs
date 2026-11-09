//! Role: Finding and reading the discovery files of the DevTools agents.

use std::fs;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use kivact_devtools_protocol::{DiscoveryFile, discovery_dir_from};

/// How long a probe of a discovery file waits for a connection. A refused loopback
/// connection takes about 2 s on Windows, so a shorter limit would mistake it for a timeout.
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// The directory that holds the discovery files of this user.
pub fn discovery_dir() -> PathBuf {
    discovery_dir_from(|name| std::env::var_os(name), std::env::temp_dir())
}

/// The reachable targets, oldest first. A file whose port refuses the connection is deleted.
/// Every file is probed on its own thread, so one slow probe does not delay the others.
pub fn list_targets() -> Vec<(PathBuf, DiscoveryFile)> {
    list_in(&discovery_dir())
}

/// The reachable targets among the discovery files of `directory`, oldest first.
fn list_in(directory: &Path) -> Vec<(PathBuf, DiscoveryFile)> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let candidates: Vec<(PathBuf, DiscoveryFile)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .filter_map(|path| {
            let file = read_target(&path).ok()?;
            Some((path, file))
        })
        .collect();
    let probed: Vec<(PathBuf, DiscoveryFile, Probe)> = std::thread::scope(|scope| {
        let handles: Vec<_> = candidates
            .into_iter()
            .map(|(path, file)| {
                scope.spawn(move || {
                    let outcome = probe(file.port);
                    (path, file, outcome)
                })
            })
            .collect();
        // A probe thread that panicked leaves its file alone, like an unknown outcome.
        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .collect()
    });
    let mut targets: Vec<(PathBuf, DiscoveryFile)> = Vec::new();
    for (path, file, outcome) in probed {
        match outcome {
            Probe::Reachable => targets.push((path, file)),
            Probe::Refused => {
                let _ = fs::remove_file(&path);
            }
            Probe::Unknown => {}
        }
    }
    targets.sort_by_key(|(_, file)| file.started_unix_ms);
    targets
}

/// Read one discovery file.
pub fn read_target(path: &Path) -> std::io::Result<DiscoveryFile> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(std::io::Error::other)
}

/// The discovery file named by `KIVACT_DEVTOOLS_TARGET`.
pub fn target_from_env() -> Option<PathBuf> {
    std::env::var_os("KIVACT_DEVTOOLS_TARGET")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// What a connection attempt to a target's port found.
enum Probe {
    /// The port accepted the connection.
    Reachable,
    /// The port refused the connection: the agent is gone.
    Refused,
    /// The attempt failed for another reason (a timeout, for example).
    Unknown,
}

fn probe(port: u32) -> Probe {
    let Ok(port) = u16::try_from(port) else {
        return Probe::Unknown;
    };
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    match TcpStream::connect_timeout(&address, PROBE_TIMEOUT) {
        Ok(_) => Probe::Reachable,
        Err(error) if error.kind() == ErrorKind::ConnectionRefused => Probe::Refused,
        Err(_) => Probe::Unknown,
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for discovery file probing.

    use std::fs;
    use std::net::TcpListener;

    use kivact_devtools_protocol::DiscoveryFile;

    use super::list_in;

    #[test]
    fn a_file_whose_port_refuses_the_connection_is_deleted() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("invariant: a loopback port can be bound in tests");
        let port = listener
            .local_addr()
            .expect("invariant: a bound listener has an address")
            .port();
        drop(listener);
        let directory = std::env::temp_dir().join(format!(
            "kivact-discovery-test-{}-{port}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("invariant: the temp directory is writable");
        let path = directory.join(format!("{}-{port}.json", std::process::id()));
        let file = DiscoveryFile {
            protocol: 1,
            pid: u64::from(std::process::id()),
            app_name: "stale".to_owned(),
            kivact_version: "test".to_owned(),
            port: u32::from(port),
            token: "token".to_owned(),
            started_unix_ms: 0,
            workspace_root: String::new(),
        };
        let text = serde_json::to_string(&file).expect("invariant: the file serializes");
        fs::write(&path, text).expect("invariant: the temp directory is writable");

        let targets = list_in(&directory);

        assert!(targets.is_empty());
        assert!(!path.exists());
        fs::remove_dir_all(&directory).expect("invariant: the test directory is removable");
    }
}
