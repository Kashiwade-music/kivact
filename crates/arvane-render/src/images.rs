//! Role: Converted images kept between frames, keyed by their pixels identity.
//!
//! vello_cpu copies an image's premultiplied pixels into a new pixmap
//! whenever it is made a paint, and the GPU renderer draws images from
//! textures, so a photo on screen would be converted (and uploaded) again on
//! every frame. The frame cache (`frame_cache.rs`) keeps each converted image
//! while consecutive frames draw it.
//!
//! Invariants (the frame cache's rules):
//! - An image is keyed by the identity of its pixel allocation (the
//!   `Arc<[u8]>` address) and its dimensions. The entry holds a clone of that
//!   `Arc` ([`Pinned`]), so the address cannot be reused by other pixels while
//!   the entry lives.
//! - An image not drawn in a frame is dropped when that frame ends. At most
//!   [`MAX_BYTES`] of converted pixels are kept; an image beyond that is
//!   converted for its frame and not kept.
//! - Thread: owned by its renderer, used on the renderer's thread.

use std::sync::Arc;

use kivact_types::pixel::Pixels;

use crate::frame_cache::{FrameCache, FrameScope, Pinned};

/// The converted pixels kept at most (256 MiB, as the facade's decoded
/// image cache).
pub(crate) const MAX_BYTES: usize = 256 * 1024 * 1024;

/// The address of the pixels and the dimensions.
pub(crate) type ImageKey = (usize, u32, u32);

/// Converted images by the identity of their pixels (see the module
/// invariants).
pub(crate) type ImageCache<V> = FrameCache<ImageKey, Pinned<V>>;

/// The converted images of the frame being drawn.
pub(crate) type ImageScope<'a, V> = FrameScope<'a, ImageKey, Pinned<V>>;

/// The identity of `image`'s pixels and its dimensions.
pub(crate) fn image_key(image: &Pixels) -> ImageKey {
    (
        Arc::as_ptr(image.rgba.as_shared()).cast::<u8>() as usize,
        image.width,
        image.height,
    )
}

/// The converted `image`: the one kept from an earlier frame, else
/// `convert`'s (kept while it fits).
pub(crate) fn converted<V: Clone>(
    scope: &mut ImageScope<'_, V>,
    image: &Pixels,
    convert: impl FnOnce() -> V,
) -> V {
    let pixels = image.rgba.as_shared();
    scope
        .get_or_insert_with(image_key(image), || {
            let pinned = Pinned::new(convert(), Arc::clone(pixels));
            (pinned, image.rgba.len())
        })
        .value
}
