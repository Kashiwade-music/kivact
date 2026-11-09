//! Role: The discovery file of this agent: writing it and removing it.

use std::path::{Path, PathBuf};

use kivact_devtools_protocol::{DiscoveryFile, discovery_dir_from, discovery_file_name};

use crate::files::write_atomic;

pub(crate) fn write(file: &DiscoveryFile) -> std::io::Result<PathBuf> {
    let dir = discovery_dir_from(|name| std::env::var_os(name), std::env::temp_dir());
    let path = dir.join(discovery_file_name(file.pid, file.port));
    let text = serde_json::to_string_pretty(file).expect("invariant: protocol values serialize");
    write_atomic(&path, text.as_bytes())?;
    Ok(path)
}

pub(crate) fn remove(path: &Path) {
    let _ = std::fs::remove_file(path);
}
