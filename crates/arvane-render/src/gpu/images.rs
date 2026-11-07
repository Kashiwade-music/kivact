//! Role: The GPU renderer's image textures and pattern atlas slots.

use crate::images::ImageCache;
use std::collections::HashMap;
use std::sync::Arc;
use vello_common::paint::ImageId;
use vello_hybrid::{TextureBindings, TextureId};

/// An image's texture.
#[derive(Clone)]
pub(crate) struct GpuImage {
    /// The renderer's id of the texture.
    pub(crate) id: TextureId,
    /// The texture's view, bound for the frames that draw it.
    pub(crate) view: wgpu::TextureView,
    /// The image's pixel width.
    pub(crate) width: u16,
    /// The image's pixel height.
    pub(crate) height: u16,
}

/// A pattern image's atlas slot.
pub(crate) struct AtlasImage {
    /// Keeps the address of the key from being reused.
    pub(crate) _pixels: Arc<[u8]>,
    /// The renderer's id of the slot.
    pub(crate) id: ImageId,
    /// The slot's pixel size (the image's, or smaller when averaged down).
    pub(crate) width: u16,
    /// The slot's pixel height.
    pub(crate) height: u16,
    /// The last frame that drew it.
    pub(crate) frame: u64,
}

/// The images the frames draw, and their bindings for the frame being
/// drawn.
pub(crate) struct Images {
    /// Textures of images, kept while frames draw them.
    pub(crate) cache: ImageCache<GpuImage>,
    /// The textures bound for the frame being drawn.
    pub(crate) bindings: TextureBindings,
    /// The last texture id handed out.
    pub(crate) next: u64,
    /// Pattern images in the atlas, by their pixels' identity and size.
    pub(crate) atlas: HashMap<(usize, u32, u32), AtlasImage>,
}

impl Images {
    /// Empty image state.
    pub(crate) fn new() -> Self {
        Self {
            cache: ImageCache::new(),
            bindings: TextureBindings::new(),
            next: 0,
            atlas: HashMap::new(),
        }
    }
}
