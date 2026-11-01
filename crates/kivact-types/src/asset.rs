//! Role: What an image or a font file is read from.
//!
//! Invariants:
//! - An [`Asset`] compares and hashes by identity, never by content: a path
//!   by the path, embedded bytes by address and length, shared bytes by the
//!   `Arc` allocation. Equal assets are the same cache entry;
//!   comparing two assets costs the same whatever their size.
//! - Holding an `Asset::Shared` keeps its allocation alive, so its address
//!   is not reused by other bytes while a cache holds the asset as a key.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// An image or a font file: bytes embedded in the executable
/// (`include_bytes!("logo.svg")`), bytes obtained at run time (`Vec<u8>`,
/// `Arc<[u8]>`), or a file read at run time (`"cover.jpg"`, a `String`, a
/// `PathBuf`, a `&Path`). Every property that takes an image accepts these
/// directly; a closure returns an `Asset` (`move || Asset::from(path.get())`).
///
/// Two assets are equal when they are the same source: the same path, the
/// same embedded bytes (address and length), or the same `Arc` allocation
/// (not equal content). The image cache keys on the same identity, so:
///
/// - Create an `Asset` from run-time bytes once, outside closures that
///   re-run (a `draw` closure, a closure passed to `image`), and clone it
///   there: clones are the same asset. `Asset::from(bytes.clone())` with a
///   `Vec<u8>` makes a new allocation, hence a new asset, every time; each
///   is decoded again, and on hosts that decode on a worker thread a
///   canvas redrawn every frame never shows its image. An `Arc<[u8]>` you
///   keep and clone is the same asset too.
/// - Name embedded bytes with a `static` (or write the `include_bytes!`
///   expression where it is used), not a `const`: a `const` is copied into
///   every use, and Rust does not promise one address for the copies, so
///   the same file may be decoded more than once.
///
/// ```ignore
/// static LOGO: &[u8] = include_bytes!("../assets/logo.svg");
/// image(LOGO);
/// image("photos/cover.jpg");
/// App::new("Editor").font(include_bytes!("../assets/Inter-Regular.ttf"));
/// ```
#[derive(Clone)]
#[non_exhaustive]
pub enum Asset {
    /// A file read at run time. A relative path resolves against the current
    /// directory, as in `std`.
    Path(PathBuf),
    /// Bytes embedded in the executable (`include_bytes!`), keyed by address
    /// and length, never copied.
    Static(&'static [u8]),
    /// Bytes obtained at run time, keyed by the `Arc` allocation.
    Shared(Arc<[u8]>),
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Asset::Path(a), Asset::Path(b)) => a == b,
            (Asset::Static(a), Asset::Static(b)) => a.as_ptr() == b.as_ptr() && a.len() == b.len(),
            (Asset::Shared(a), Asset::Shared(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for Asset {}

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Asset::Path(p) => {
                state.write_u8(0);
                p.hash(state);
            }
            Asset::Static(b) => {
                state.write_u8(1);
                (b.as_ptr() as usize).hash(state);
                b.len().hash(state);
            }
            Asset::Shared(b) => {
                state.write_u8(2);
                (b.as_ptr() as usize).hash(state);
                b.len().hash(state);
            }
        }
    }
}

impl fmt::Debug for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Asset::Path(p) => f.debug_tuple("Path").field(p).finish(),
            Asset::Static(b) => write!(f, "Static({} bytes at {:p})", b.len(), b.as_ptr()),
            Asset::Shared(b) => write!(f, "Shared({} bytes at {:p})", b.len(), b.as_ptr()),
        }
    }
}

impl<const N: usize> From<&'static [u8; N]> for Asset {
    /// Bytes embedded with `include_bytes!` (not copied).
    fn from(bytes: &'static [u8; N]) -> Asset {
        Asset::Static(bytes)
    }
}

impl From<&'static [u8]> for Asset {
    /// Bytes embedded in the executable (not copied).
    fn from(bytes: &'static [u8]) -> Asset {
        Asset::Static(bytes)
    }
}

impl From<Vec<u8>> for Asset {
    /// Bytes obtained at run time (a file read by the app, an HTTP body).
    fn from(bytes: Vec<u8>) -> Asset {
        Asset::Shared(Arc::from(bytes))
    }
}

impl From<Arc<[u8]>> for Asset {
    /// Bytes obtained at run time, shared (equal while the same `Arc`).
    fn from(bytes: Arc<[u8]>) -> Asset {
        Asset::Shared(bytes)
    }
}

impl From<&'static str> for Asset {
    /// A file read at run time.
    fn from(path: &'static str) -> Asset {
        Asset::Path(PathBuf::from(path))
    }
}

impl From<String> for Asset {
    /// A file read at run time.
    fn from(path: String) -> Asset {
        Asset::Path(PathBuf::from(path))
    }
}

impl From<PathBuf> for Asset {
    /// A file read at run time.
    fn from(path: PathBuf) -> Asset {
        Asset::Path(path)
    }
}

impl From<&Path> for Asset {
    /// A file read at run time.
    fn from(path: &Path) -> Asset {
        Asset::Path(path.to_owned())
    }
}
