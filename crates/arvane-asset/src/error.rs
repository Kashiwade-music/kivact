//! Role: Asset failures and their messages.
//!
//! Invariants:
//! - A message starts with the source: for a file, its resolved absolute
//!   path and the current directory (a relative path resolves against the
//!   current directory, so both are needed to see why it was not found);
//!   for bytes, their kind and length. The reason follows.
//! - Errors compare by kind and message, so a cached failure equals the
//!   failure it was made from.

use std::fmt;
use std::path::Path;

use ui_types::Asset;

/// Why an asset failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetErrorKind {
    /// The file does not exist.
    NotFound,
    /// The file exists but could not be read.
    Io,
    /// A path that is a URL (`http://…`); URLs are not loaded.
    Url,
    /// Over a limit: encoded size, decoded edge or the cache budget.
    TooLarge,
    /// The content is not a supported image format.
    UnsupportedFormat,
    /// The content looks like a supported format but does not decode.
    Decode,
    /// Not a usable font file (or a WOFF/WOFF2 one).
    Font,
}

/// A failed asset: the kind and a message naming the source and the reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetError {
    kind: AssetErrorKind,
    message: String,
}

impl AssetError {
    /// An error for `asset` with `reason` appended to its source description.
    pub(crate) fn new(kind: AssetErrorKind, asset: &Asset, reason: impl fmt::Display) -> Self {
        AssetError {
            kind,
            message: format!("{}: {reason}", describe(asset)),
        }
    }

    /// Why it failed.
    pub fn kind(&self) -> AssetErrorKind {
        self.kind
    }

    /// The full message (also the `Display` form).
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AssetError {}

/// How a message names an asset.
fn describe(asset: &Asset) -> String {
    match asset {
        Asset::Path(p) if crate::source::is_url(p) => format!("URL `{}`", p.display()),
        Asset::Path(p) => {
            let cwd = std::env::current_dir()
                .map_or_else(|e| format!("unknown: {e}"), |d| d.display().to_string());
            format!(
                "file `{}` (current directory `{cwd}`)",
                absolute(p).display()
            )
        }
        Asset::Static(b) => format!("embedded bytes ({} bytes)", b.len()),
        Asset::Shared(b) => format!("bytes ({} bytes)", b.len()),
    }
}

/// `p` resolved against the current directory (as the file system resolves
/// it), or `p` itself when that fails.
fn absolute(p: &Path) -> std::path::PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_owned())
}
