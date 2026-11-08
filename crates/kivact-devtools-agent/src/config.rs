//! Role: The workspace root and the `.kivact/devtools.json` configuration.

use std::path::{Path, PathBuf};

use kivact_devtools_protocol::DevtoolsConfig;

/// From the current directory upward: the first directory with `.kivact`, else the first with
/// `Cargo.lock`. Neither: the same search from the executable's directory (an editor embedded in
/// a DAW has the DAW's cwd). Neither: the current directory. Nothing is created here.
pub(crate) fn find_workspace_root() -> PathBuf {
    let cwd = std::env::current_dir().ok();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    cwd.as_deref()
        .and_then(search)
        .or_else(|| exe_dir.as_deref().and_then(search))
        .or(cwd)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(crate) fn search(start: &Path) -> Option<PathBuf> {
    let marked = |name: &str| {
        start
            .ancestors()
            .find(|dir| dir.join(name).exists())
            .map(Path::to_path_buf)
    };
    marked(".kivact").or_else(|| marked("Cargo.lock"))
}

/// `.kivact/devtools.json`; a missing file is the default, an unreadable or invalid one is the
/// default plus a warning.
pub(crate) fn load_config(root: &Path) -> DevtoolsConfig {
    let path = root.join(".kivact").join("devtools.json");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return DevtoolsConfig::default();
        }
        Err(error) => {
            tracing::warn!(target: "kivact::devtools", "cannot read {}: {error}", path.display());
            return DevtoolsConfig::default();
        }
    };
    serde_json::from_str(&text).unwrap_or_else(|error| {
        tracing::warn!(target: "kivact::devtools", "invalid {}: {error}", path.display());
        DevtoolsConfig::default()
    })
}
