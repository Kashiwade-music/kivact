//! Role: Loading and saving the board as a JSON file.

use crate::model::Board;
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
        .join("KivactExamples")
        .join("task-board.json")
}

const MAX_BYTES: u64 = 64 * 1024 * 1024;

/// Load the saved board. When the file does not exist and `generated` is set,
/// start from an unsaved board of that many generated tasks (at most 100,000).
pub fn load_initial(path: &Path, generated: Option<usize>) -> Result<Board, String> {
    match generated {
        Some(count) if !path.exists() => Ok(Board::generated(count.min(100_000))),
        _ => load(path),
    }
}

pub fn load(path: &Path) -> Result<Board, String> {
    use std::io::Read;
    let file = match fs::File::open(path) {
        Ok(file) => file,
        // No file there yet. A file where a parent directory should be is
        // `NotFound` on Windows but `NotADirectory` on Unix; either way there
        // is no board to read (saving reports why it cannot create one).
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
            return Ok(Board::default());
        }
        Err(error) => return Err(format!("保存ファイルを読めません: {error}")),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("保存ファイルが64MiBを超えています".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| {
        // Copy, rather than rename, so a failed backup never removes the original.
        let backup = path.with_extension(format!(
            "corrupt-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        match fs::copy(path, &backup) {
            Ok(_) => format!("JSONを読めません: {error}。退避先: {}", backup.display()),
            Err(backup_error) => {
                format!("JSONを読めません: {error}。退避できません: {backup_error}")
            }
        }
    })
}

pub fn save(path: &Path, board: &Board) -> Result<(), String> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec(board).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("保存内容が64MiBを超えています".into());
    }
    let temporary = path.with_extension(format!(
        "{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let mut created = false;
    let result = (|| -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        created = true;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        crate::platform::sync_dir(
            path.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new(".")),
        )
    })();
    if result.is_err() && created {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| error.to_string())
}
