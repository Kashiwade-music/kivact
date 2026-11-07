//! Role: Converted images kept from one frame to the next.
//!
//! Converted images kept from one frame to the next: vello_cpu copies an
//! image's straight-alpha pixels into a new premultiplied pixmap whenever
//! it is made a paint, and the GPU renderer draws images from textures, so
//! a photo on screen would be converted (and uploaded) again on every
//! frame.
//!
//! Invariants (the frame cache rules):
//! - An entry is keyed by the identity of the image's pixel allocation (the
//!   `Arc<[u8]>` address) and its dimensions, and keeps a clone of that
//!   `Arc`, so the address cannot be reused by other pixels while the entry
//!   lives.
//! - Bounded by the scene: an entry not drawn in a frame is dropped when
//!   that frame ends, so the cache holds at most the images of the last
//!   frame, and at most `MAX_BYTES` of converted pixels (an image beyond
//!   that is converted for its frame and not kept).
//! - Thread: owned by its renderer, used on the renderer's thread.

use std::collections::HashMap;
use std::sync::Arc;

use kivact_types::pixel::Pixels;

/// The converted pixels kept at most (256 MiB, as the facade's decoded
/// image cache).
const MAX_BYTES: usize = 256 * 1024 * 1024;

/// The address of the pixels and the dimensions.
type Key = (usize, u32, u32);

struct Entry<V> {
    /// Keeps the address of the key from being reused.
    _pixels: Arc<[u8]>,
    value: V,
    bytes: usize,
    /// The last frame that drew it.
    frame: u64,
}

/// Converted images by the identity of their pixels (see the module
/// invariants).
pub(crate) struct ImageCache<V> {
    entries: HashMap<Key, Entry<V>>,
    frame: u64,
    bytes: usize,
}

impl<V: Clone> ImageCache<V> {
    pub(crate) fn new() -> Self {
        ImageCache {
            entries: HashMap::new(),
            frame: 0,
            bytes: 0,
        }
    }

    /// Starts a frame.
    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;
    }

    /// The frame being drawn (counted by `begin_frame`), which stamps what
    /// it uses.
    #[cfg(feature = "gpu")]
    pub(crate) fn frame(&self) -> u64 {
        self.frame
    }

    /// The converted `image`: the one kept from an earlier frame, else
    /// `convert`'s (kept while it fits).
    pub(crate) fn get(&mut self, image: &Pixels, convert: impl FnOnce() -> V) -> V {
        let key = (
            Arc::as_ptr(image.rgba.as_shared()).cast::<u8>() as usize,
            image.width,
            image.height,
        );
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.frame = self.frame;
            return entry.value.clone();
        }
        let value = convert();
        let bytes = image.rgba.len();
        if self.bytes + bytes <= MAX_BYTES {
            self.bytes += bytes;
            self.entries.insert(
                key,
                Entry {
                    _pixels: Arc::clone(image.rgba.as_shared()),
                    value: value.clone(),
                    bytes,
                    frame: self.frame,
                },
            );
        }
        value
    }

    /// Ends the frame: drops the images it did not draw.
    pub(crate) fn end_frame(&mut self) {
        let frame = self.frame;
        let mut freed = 0;
        self.entries.retain(|_, e| {
            let keep = e.frame == frame;
            if !keep {
                freed += e.bytes;
            }
            keep
        });
        self.bytes -= freed;
    }
}
