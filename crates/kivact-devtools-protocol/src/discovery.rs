//! Role: Where the DevTools discovery files live; the caller passes the environment.

use std::ffi::OsString;
use std::path::PathBuf;

/// The discovery directory: `%LOCALAPPDATA%\kivact\devtools`, else `$XDG_RUNTIME_DIR/kivact/devtools`,
/// else `$HOME/.cache/kivact/devtools`, else `<temp_dir>/kivact-devtools`. Empty values count as unset.
/// Callers pass `|name| std::env::var_os(name)` and `std::env::temp_dir()`.
pub fn discovery_dir_from(var: impl Fn(&str) -> Option<OsString>, temp_dir: PathBuf) -> PathBuf {
    let set = |name: &str| var(name).filter(|value| !value.is_empty());
    if let Some(base) = set("LOCALAPPDATA") {
        return PathBuf::from(base).join("kivact").join("devtools");
    }
    if let Some(base) = set("XDG_RUNTIME_DIR") {
        return PathBuf::from(base).join("kivact").join("devtools");
    }
    if let Some(home) = set("HOME") {
        return PathBuf::from(home)
            .join(".cache")
            .join("kivact")
            .join("devtools");
    }
    temp_dir.join("kivact-devtools")
}

/// `<pid>-<port>.json`.
pub fn discovery_file_name(pid: u64, port: u32) -> String {
    format!("{pid}-{port}.json")
}
