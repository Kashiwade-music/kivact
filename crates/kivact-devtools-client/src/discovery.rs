//! Role: Finding and reading the discovery files of the DevTools agents.

use std::fs;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use kivact_devtools_protocol::{DiscoveryFile, discovery_dir_from};

/// How long a probe of a discovery file waits for a connection.
const PROBE_TIMEOUT: Duration = Duration::from_millis(100);

/// The directory that holds the discovery files of this user.
pub fn discovery_dir() -> PathBuf {
    discovery_dir_from(|name| std::env::var_os(name), std::env::temp_dir())
}

/// The reachable targets, oldest first. A file whose port refuses the connection is deleted.
pub fn list_targets() -> Vec<(PathBuf, DiscoveryFile)> {
    let Ok(entries) = fs::read_dir(discovery_dir()) else {
        return Vec::new();
    };
    let mut targets: Vec<(PathBuf, DiscoveryFile)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Ok(file) = read_target(&path) else {
            continue;
        };
        match probe(file.port) {
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
