//! Role: Platform differences of the board storage.

use std::path::Path;

/// Makes a rename inside `dir` durable. Unix needs the directory itself
/// synced; on Windows the rename is durable once the file is synced.
pub(crate) fn sync_dir(dir: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    std::fs::File::open(dir)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}
