//! Role: The asset store: one per host instance.
//! It loads images on its worker
//! thread (or inline), keeps decoded pixels in an LRU within a byte budget,
//! and answers what to draw.
//!
//! Invariants:
//! - One cache entry per [`Asset`] (by identity: a path, embedded bytes by
//!   address and length, shared bytes by allocation; the entry's key keeps
//!   a shared allocation alive so its address is not reused). Mounting the
//!   same asset any number of times loads it once. A relative path is
//!   keyed by its absolute form against the current directory at the call
//!   ([`key`](asset_store::key)(asset_store::key)), so after the current directory changes it names another
//!   file and another entry (std's meaning of a relative path); the assets
//!   [`AssetStore::take_failures`] returns are these keys.
//! - Failures are cached like images. A failed file is loaded again only
//!   when its length or modification time changes, checked at most once
//!   per `RECHECK_INTERVAL` and on the worker (the check reads metadata,
//!   which blocks on an unreachable network path; an inline store checks
//!   in the call, still throttled). A URL failure is never checked again.
//!   A loaded file is not watched (automatic reload is out of scope).
//! - The budget counts decoded pixels (sources, SVG rasters, tinted
//!   variants), the parsed SVG trees (estimated), the encoded bytes entry
//!   keys keep alive and a fixed cost per entry (`entry::ENTRY_BYTES`).
//!   Recency is per frame: the host calls [`AssetStore::begin_frame`]
//!   before a pass that draws every image it shows, and what the current
//!   and the previous frame used is never evicted (`evict`). Pixels larger
//!   than the whole budget are not kept: a decoded image that large fails
//!   as `TooLarge`.
//! - Asking for a natural size never starts work: [`AssetStore::peek`]
//!   reads the cache only, and a raster source whose pixels were evicted
//!   keeps its natural size (its pixels are loaded again when drawn).
//! - Every evicted or replaced pixel id is reported once through
//!   [`AssetStore::take_evicted`]; every failure once through
//!   [`AssetStore::take_failures`].
//! - [`AssetStore::pixels`] never returns pixels of another asset. While a
//!   new SVG raster of an asset is pending, it returns that asset's most
//!   recent raster (scaled by the host), never an empty frame. A raster
//!   that lands is stamped with the frame its request was last made in, so
//!   a size no longer asked for is evictable at once (and dropped when the
//!   asset has another raster to fall back on).
//! - Results of jobs for entries evicted or replaced meanwhile are dropped
//!   (each entry has a generation).
//! - Thread: the store lives on its host's UI thread. Tints are computed
//!   on the calling thread (one pass over the pixels, cached per color);
//!   loads, decodes and rasters run on the worker in worker mode.

mod asset_store;
mod entry;
mod evict;
pub(crate) mod image_state;
mod job;
mod worker;

pub use asset_store::{AssetStore, DEFAULT_BUDGET};
