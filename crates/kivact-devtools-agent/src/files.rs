//! Role: Atomic file writes and the default path of a trace file.

use std::path::{Path, PathBuf};

/// Create the parent directories, write `<path>.tmp`, then rename it over `path`.
pub(crate) fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    std::fs::write(&temp, contents)?;
    std::fs::rename(&temp, path)
}

/// `requested` is relative to `root` (an absolute path stays as it is); `None` is
/// `.kivact/traces/<session>-<n>.json` with the first `n` from 1 whose file does not exist yet.
pub(crate) fn trace_path(root: &Path, session: &str, requested: Option<&str>) -> PathBuf {
    if let Some(path) = requested {
        return root.join(path);
    }
    let mut number = 1_u32;
    loop {
        let path = root
            .join(".kivact")
            .join("traces")
            .join(format!("{session}-{number}.json"));
        if !path.exists() {
            return path;
        }
        number += 1;
    }
}
