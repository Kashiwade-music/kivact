//! Role: Reading an asset's encoded bytes from memory or from files on disk.
//!
//! Reading an asset's encoded bytes: embedded and shared bytes as they are,
//! files from disk within the 64 MiB limit.
//!
//! Invariants:
//! - Embedded and shared bytes are never copied.
//! - A file is read at most `MAX_ENCODED_BYTES + 1` bytes far, so a huge or
//!   growing file cannot exhaust memory; over the limit is `TooLarge`.
//! - A path that looks like a URL (`scheme://…`) is not opened.

use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use crate::source::is_url;
use ui_types::Asset;

use crate::error::{AssetError, AssetErrorKind};
use crate::limits::MAX_ENCODED_BYTES;

/// Encoded bytes of an asset.
#[derive(Clone)]
pub(crate) enum Encoded {
    Static(&'static [u8]),
    Shared(Arc<[u8]>),
}

impl std::ops::Deref for Encoded {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            Encoded::Static(b) => b,
            Encoded::Shared(b) => b,
        }
    }
}

/// A file's length and modification time: a file is read again only after
/// either changes. `None` when the file cannot be inspected.
pub(crate) type FileVersion = Option<(u64, Option<SystemTime>)>;

/// The current version of the file at `path`.
pub(crate) fn file_version(path: &Path) -> FileVersion {
    std::fs::metadata(path)
        .ok()
        .map(|m| (m.len(), m.modified().ok()))
}

/// The encoded bytes of `asset`.
pub(crate) fn read(asset: &Asset) -> Result<Encoded, AssetError> {
    match asset {
        Asset::Static(b) => Ok(Encoded::Static(b)),
        Asset::Shared(b) => Ok(Encoded::Shared(Arc::clone(b))),
        Asset::Path(p) => read_file(asset, p).map(Encoded::Shared),
    }
}

fn read_file(asset: &Asset, path: &Path) -> Result<Arc<[u8]>, AssetError> {
    if is_url(path) {
        return Err(AssetError::new(
            AssetErrorKind::Url,
            asset,
            "URLs are not supported; fetch the bytes yourself and pass them as `Vec<u8>`",
        ));
    }
    let io = |e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AssetError::new(AssetErrorKind::NotFound, asset, "not found")
        } else {
            AssetError::new(AssetErrorKind::Io, asset, format!("cannot read: {e}"))
        }
    };
    let file = std::fs::File::open(path).map_err(io)?;
    let mut bytes = Vec::new();
    file.take(MAX_ENCODED_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes.len() as u64 > MAX_ENCODED_BYTES {
        return Err(too_large(asset));
    }
    Ok(bytes.into())
}

/// The error for encoded input over `MAX_ENCODED_BYTES`.
pub(crate) fn too_large(asset: &Asset) -> AssetError {
    AssetError::new(
        AssetErrorKind::TooLarge,
        asset,
        format!(
            "larger than the {} MiB limit for encoded input",
            MAX_ENCODED_BYTES / (1024 * 1024)
        ),
    )
}
