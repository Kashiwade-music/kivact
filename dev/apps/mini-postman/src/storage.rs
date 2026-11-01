//! Role: Durable request history shared by independent app processes.
use crate::http::Request;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub(crate) const HISTORY_LIMIT: usize = 30;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub request: Request,
    pub result: String,
}

pub fn history_path() -> PathBuf {
    if let Some(path) = std::env::var_os("KIVACT_POSTMAN_HISTORY") {
        return PathBuf::from(path);
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("kivact"));
    base.join("kivact")
        .join("mini-postman")
        .join("history.json")
}

pub fn load(path: &Path) -> io::Result<Vec<HistoryEntry>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut entries: Vec<HistoryEntry> = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    entries.truncate(HISTORY_LIMIT);
    Ok(entries)
}

pub fn save(path: &Path, entries: &[HistoryEntry]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("history path has no parent"))?;
    fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(&entries[..entries.len().min(HISTORY_LIMIT)])?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = fs::File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}
