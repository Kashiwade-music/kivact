//! Role: Decoded straight-alpha images that scenes draw.

use std::sync::Arc;

/// Straight-alpha RGBA8 pixels, usable on any thread: a decoded image, or
/// raw pixels a host makes itself (a scope, a color grading view) without
/// any decoding.
/// Pixels are row-major from the top left, four bytes per pixel in RGBA
/// order, with no row padding. These are source pixels, independent of the
/// logical destination rectangle and window scale.
///
/// Renderers know an image by the identity of its `rgba` allocation (the
/// `Arc`), not its content: pixels drawn in consecutive frames are
/// converted (premultiplied, uploaded to a GPU texture) once and kept
/// while frames draw them, and dropped after the first frame that does
/// not. Pixels that change every frame are a new `Arc` every frame (the
/// previous one's conversion is dropped after the next frame, so nothing
/// accumulates); unchanged pixels should keep their `Arc`.
///
/// ```
/// use kivact_types::protocol::ImageData;
/// use std::sync::Arc;
/// // A 2 × 1 image: opaque red, half-transparent blue.
/// let image = ImageData {
///     width: 2,
///     height: 1,
///     rgba: Arc::from(&[255u8, 0, 0, 255, 0, 0, 255, 128][..]),
/// };
/// assert_eq!(image.rgba.len(), 2 * 1 * 4);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ImageData {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Exactly width * height * 4 straight-alpha RGBA8 bytes, row-major.
    pub rgba: Arc<[u8]>,
}
