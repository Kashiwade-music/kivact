//! Role: Defines asset loading and rasterization limits.

/// Encoded input larger than this fails (64 MiB).
pub const MAX_ENCODED_BYTES: u64 = 64 * 1024 * 1024;
/// A decoded raster image with a longer edge fails (16,384 px).
pub const MAX_IMAGE_EDGE: u32 = 16_384;
/// SVG rasters are at most this many device pixels per edge (8,192 px).
pub const MAX_SVG_RASTER_EDGE: u32 = 8_192;

/// What an entry costs besides its pixels and tree: the key, the entry and
/// its map slot, roughly. Counting it keeps entries that hold no pixels
/// (failures, evicted sources) within the budget too.
pub(crate) const ENTRY_BYTES: usize = 256;
