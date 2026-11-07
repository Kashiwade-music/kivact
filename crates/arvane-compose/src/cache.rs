//! Role: Previous frame's chunk and layer records: the plan's pure cache index.
//!
//! The plan only reads this index and returns the records of the frame it
//! planned; the executor stores them after a successful frame.

use std::collections::HashMap;
use std::sync::Arc;

use kivact_types::AssetBytes;
use kivact_types::geometry::Rect;
use kivact_types::render::GlyphRunSnapshot;
use kivact_types::render::{LayerKey, PixelRect};

/// What a chunk of a layer holds (the key's second half).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ChunkRole {
    /// The n-th Raster item of the layer.
    Content(u32),
    /// Clips of the n-th item (LayerItem::Layer / External).
    Clip(u32),
    /// The layer's own clip (LayerEffects::clip).
    LayerClip,
    /// The n-th RenderMask.
    Mask(u32),
    /// The coverage of the n-th Shader item.
    Coverage(u32),
    /// The fallback of the n-th Shader item.
    Fallback(u32),
    /// The backdrop shape.
    Backdrop,
}

/// Identity of one chunk across frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ChunkKey {
    /// The layer the chunk belongs to.
    pub(crate) layer: LayerKey,
    /// Which chunk of that layer.
    pub(crate) role: ChunkRole,
}

/// A shared allocation a cached record must keep alive, so that the
/// address folded into its hash is not reused for other contents.
#[derive(Clone, Debug)]
#[expect(dead_code, reason = "held so the hashed pointers stay valid (§8.6)")]
pub(crate) enum Retained {
    /// Pixel or byte storage.
    Bytes(AssetBytes),
    /// A shaped glyph run.
    Glyphs(GlyphRunSnapshot),
}

/// The previous frame's record of one chunk.
#[derive(Clone, Debug)]
pub(crate) struct ChunkRecord {
    /// Device px per layer unit.
    pub(crate) scale: f64,
    /// Layer point of chunk px (0, 0).
    pub(crate) origin: kivact_types::geometry::Point,
    /// Size in px.
    pub(crate) size: (u32, u32),
    /// Per command: hash and chunk px bounds (None: draws nothing there).
    pub(crate) index: Arc<[(u64, Option<PixelRect>)]>,
    /// The frame that last used the record.
    pub(crate) frame: u64,
    /// Allocations the index hashes by address.
    #[expect(dead_code, reason = "held so the hashed pointers stay valid (§8.6)")]
    pub(crate) retain: Vec<Retained>,
}

/// The previous frame's record of one layer.
#[derive(Clone, Debug)]
pub(crate) struct LayerRecord {
    /// Hash of the surface's content, scale and rectangle (decides reuse).
    pub(crate) signature: u64,
    /// Hash of the layer's own drawing parameters and window mapping.
    pub(crate) own: u64,
    /// Where the layer's surface lands in the window, None when it drew nothing.
    pub(crate) window_bounds: Option<Rect>,
    /// The frame that last used the record.
    pub(crate) frame: u64,
    /// Allocations the signature hashes by address (images, shader and filter pixels).
    #[expect(dead_code, reason = "held so the hashed pointers stay valid (§8.6)")]
    pub(crate) retain: Vec<Retained>,
}

/// Frames a record may go unused before it is dropped.
const KEEP_FRAMES: u64 = 2;

/// The chunk and layer records of the frames drawn so far.
#[derive(Default)]
pub(crate) struct CacheIndex {
    /// Chunk records by key.
    pub(crate) chunks: HashMap<ChunkKey, ChunkRecord>,
    /// Layer records by key.
    pub(crate) layers: HashMap<LayerKey, LayerRecord>,
    /// Frames drawn so far (cache stamps; statistics, not a phase).
    pub(crate) frame: u64,
}

impl CacheIndex {
    /// Store the records of the frame `frame` and drop the records that
    /// were not used for [`KEEP_FRAMES`] frames.
    pub(crate) fn store(
        &mut self,
        frame: u64,
        chunks: Vec<(ChunkKey, ChunkRecord)>,
        layers: Vec<(LayerKey, LayerRecord)>,
    ) {
        self.frame = frame;
        self.chunks.extend(chunks);
        self.layers.extend(layers);
        self.chunks
            .retain(|_, record| record.frame + KEEP_FRAMES > frame);
        self.layers
            .retain(|_, record| record.frame + KEEP_FRAMES > frame);
    }

    /// Forget every record (after a resize, a lost device or a failed frame).
    pub(crate) fn clear(&mut self) {
        self.chunks.clear();
        self.layers.clear();
    }
}
