//! Role: The GPU renderer's image textures and pattern atlas slots.

use vello_common::paint::ImageId;
use vello_hybrid::{TextureBindings, TextureId};

/// The most pattern images in the atlas at once (16 MiB): the atlas is the
/// glyphs' too, and vello_hybrid panics when its pages run out. Each slot
/// costs 1 in the atlas's frame cache.
pub(crate) const MAX_ATLAS_PATTERNS: usize = 64;

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
#[derive(Clone)]
pub(crate) struct AtlasImage {
    /// The renderer's id of the slot.
    pub(crate) id: ImageId,
    /// The slot's pixel size (the image's, or smaller when averaged down).
    pub(crate) width: u16,
    /// The slot's pixel height.
    pub(crate) height: u16,
}

/// The image bindings the frames draw with.
pub(crate) struct Images {
    /// The textures bound for the frame being drawn.
    pub(crate) bindings: TextureBindings,
    /// The last texture id handed out.
    pub(crate) next: u64,
}

impl Images {
    /// Empty image state.
    pub(crate) fn new() -> Self {
        Self {
            bindings: TextureBindings::new(),
            next: 0,
        }
    }
}
