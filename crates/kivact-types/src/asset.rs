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
/// [`AssetBytes`]), or a file read at run time (`"cover.jpg"`, a `String`, a
/// `PathBuf`, a `&Path`). Every property that takes an image accepts these
/// directly; a closure returns an `Asset` (`move || Asset::from(path.get())`).
///
/// Two assets are equal when they are the same source: the same path, the
/// same embedded bytes (address and length), or the same retained byte allocation
/// (not equal content). The image cache keys on the same identity, so:
///
/// - Create an `Asset` from run-time bytes once, outside closures that
///   re-run (a `draw` closure, a closure passed to `image`), and clone it
///   there: clones are the same asset. `Asset::from(bytes.clone())` with a
///   `Vec<u8>` makes a new allocation, hence a new asset, every time; each
///   is decoded again, and on hosts that decode on a worker thread a
///   canvas redrawn every frame never shows its image. An `AssetBytes` value you
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
pub enum Asset {
    /// A file read at run time. A relative path resolves against the current
    /// directory, as in `std`.
    Path(PathBuf),
    /// Bytes embedded in the executable (`include_bytes!`), keyed by address
    /// and length, never copied.
    Static(&'static [u8]),
    /// Bytes obtained at run time, keyed by the `Arc` allocation.
    Shared(AssetBytes),
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Asset::Path(a), Asset::Path(b)) => a == b,
            (Asset::Static(a), Asset::Static(b)) => a.as_ptr() == b.as_ptr() && a.len() == b.len(),
            (Asset::Shared(a), Asset::Shared(b)) => a.same_allocation(b),
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
        Asset::Shared(bytes.into())
    }
}

impl From<AssetBytes> for Asset {
    /// Retained bytes share the same asset cache identity across clones.
    fn from(bytes: AssetBytes) -> Asset {
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

/// Immutable binary data whose clones retain one byte allocation.
/// Equality and hashing compare byte contents; `same_allocation` tests storage identity.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AssetBytes(Arc<[u8]>);
impl AssetBytes {
    /// Retain owned bytes without cloning their contents.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes.into())
    }
    /// Copy borrowed bytes once into retained storage.
    pub fn copy_from_slice(bytes: &[u8]) -> Self {
        Self(Arc::from(bytes))
    }
    /// Whether both values retain the same byte allocation.
    pub fn same_allocation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    /// Adopt backend-owned bytes without copying or changing allocation identity.
    // xtask-allow(internal-contract): Asset/font/renderer adapters retain an existing backend allocation; application inputs use owned bytes.
    #[doc(hidden)]
    pub fn from_shared(bytes: Arc<[u8]>) -> Self {
        Self(bytes)
    }
    /// Borrow the retained allocation for a backend that requires shared ownership.
    // xtask-allow(internal-contract): Asset/font/renderer adapters bridge shared backend resources without copying their bytes.
    #[doc(hidden)]
    pub fn as_shared(&self) -> &Arc<[u8]> {
        &self.0
    }
}
impl From<Vec<u8>> for AssetBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}
impl<const N: usize> From<[u8; N]> for AssetBytes {
    fn from(bytes: [u8; N]) -> Self {
        Self(Arc::from(bytes))
    }
}
impl std::ops::Deref for AssetBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}
impl AsRef<[u8]> for AssetBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
