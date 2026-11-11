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
#[expect(dead_code, reason = "held so the hashed pointers stay valid")]
pub(crate) enum Retained {
    /// Pixel or byte storage.
    Bytes(AssetBytes),
    /// A shaped glyph run.
    Glyphs(GlyphRunSnapshot),
}

/// Where one command of a chunk's index reaches, in chunk px.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Reach {
    /// Nothing of it lands in the chunk.
    Nothing,
    /// A command that draws: it damages this rectangle when it changes or
    /// moves.
    Draws(PixelRect),
    /// A group opener (a clip, a transform, a group without blur): it draws
    /// nothing of its own and damages its group's bounds only when it
    /// changed itself (its hash), never because its content's extent moved
    /// (each content command damages its own place).
    Opens(PixelRect),
}

/// One command of a chunk's index: its hash and where it reaches.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Indexed {
    pub(crate) hash: u64,
    pub(crate) reach: Reach,
}

impl Indexed {
    /// The rectangle this command draws in (`None` for an opener or a
    /// command outside the chunk): with the hash, what makes two entries
    /// the same.
    fn drawn(&self) -> Option<PixelRect> {
        match self.reach {
            Reach::Draws(rect) => Some(rect),
            Reach::Opens(_) | Reach::Nothing => None,
        }
    }

    /// What a change of this command damages.
    pub(crate) fn damage(&self) -> Option<PixelRect> {
        match self.reach {
            Reach::Draws(rect) | Reach::Opens(rect) => Some(rect),
            Reach::Nothing => None,
        }
    }
}

impl PartialEq for Indexed {
    fn eq(&self, o: &Self) -> bool {
        self.hash == o.hash && self.drawn() == o.drawn()
    }
}

impl Eq for Indexed {}

/// The previous frame's record of one chunk.
#[derive(Clone, Debug)]
pub(crate) struct ChunkRecord {
    /// Device px per layer unit.
    pub(crate) scale: f64,
    /// Layer point of chunk px (0, 0).
    pub(crate) origin: kivact_types::geometry::Point,
    /// Size in px.
    pub(crate) size: (u32, u32),
    /// Per command: its index entry.
    pub(crate) index: Arc<[Indexed]>,
    /// The frame that last used the record.
    pub(crate) frame: u64,
    /// Allocations the index hashes by address.
    #[expect(dead_code, reason = "held so the hashed pointers stay valid")]
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
    #[expect(dead_code, reason = "held so the hashed pointers stay valid")]
    pub(crate) retain: Vec<Retained>,
}

/// Frames a record may go unused before it is dropped.
const KEEP_FRAMES: u64 = 2;

/// The chunk and layer records of the frames drawn so far.
#[derive(Default)]
pub(crate) struct CacheIndex {
    /// Frames drawn so far (cache stamps; statistics, not a phase).
    pub(crate) frame: u64,
    /// The records kept from the last frame, unless the next frame redraws
    /// everything.
    pub(crate) records: Records,
}

/// What the cache keeps of the last frame.
pub(crate) enum Records {
    /// The records of the last frame, by key.
    Kept {
        /// Chunk records by key.
        chunks: HashMap<ChunkKey, ChunkRecord>,
        /// Layer records by key.
        layers: HashMap<LayerKey, LayerRecord>,
    },
    /// The next frame redraws everything: no record is kept.
    Invalidated,
}

impl Default for Records {
    fn default() -> Self {
        Self::Kept {
            chunks: HashMap::new(),
            layers: HashMap::new(),
        }
    }
}

impl Records {
    /// The kept maps, empty after [`Records::Invalidated`] (nothing is kept
    /// then).
    fn kept(
        &mut self,
    ) -> (
        &mut HashMap<ChunkKey, ChunkRecord>,
        &mut HashMap<LayerKey, LayerRecord>,
    ) {
        if matches!(self, Self::Invalidated) {
            *self = Self::default();
        }
        match self {
            Self::Kept { chunks, layers } => (chunks, layers),
            Self::Invalidated => unreachable!("just made Kept"),
        }
    }
}

impl CacheIndex {
    /// Whether the chunk record `key` is kept: false after `clear`, when the
    /// next frame redraws everything.
    pub(crate) fn keeps_chunk(&self, key: &ChunkKey) -> bool {
        match &self.records {
            Records::Kept { chunks, .. } => chunks.contains_key(key),
            Records::Invalidated => false,
        }
    }

    /// Whether the layer record `key` is kept: false after `clear`, when the
    /// next frame redraws everything.
    pub(crate) fn keeps_layer(&self, key: &LayerKey) -> bool {
        match &self.records {
            Records::Kept { layers, .. } => layers.contains_key(key),
            Records::Invalidated => false,
        }
    }

    /// The chunk record `key`, when the cache keeps one.
    pub(crate) fn chunk(&self, key: &ChunkKey) -> Option<&ChunkRecord> {
        match &self.records {
            Records::Kept { chunks, .. } => chunks.get(key),
            Records::Invalidated => None,
        }
    }

    /// The layer record `key`, when the cache keeps one.
    pub(crate) fn layer(&self, key: &LayerKey) -> Option<&LayerRecord> {
        match &self.records {
            Records::Kept { layers, .. } => layers.get(key),
            Records::Invalidated => None,
        }
    }

    /// Store the records of the frame `frame` and drop the records that
    /// were not used for [`KEEP_FRAMES`] frames. `forgets` are chunks the
    /// frame drew without caching: their old records no longer match the
    /// pixels under their keys, so they go first.
    pub(crate) fn store(
        &mut self,
        frame: u64,
        chunks: Vec<(ChunkKey, ChunkRecord)>,
        layers: Vec<(LayerKey, LayerRecord)>,
        forgets: &[ChunkKey],
    ) {
        self.frame = frame;
        let (kept_chunks, kept_layers) = self.records.kept();
        for key in forgets {
            kept_chunks.remove(key);
        }
        kept_chunks.extend(chunks);
        kept_layers.extend(layers);
        kept_chunks.retain(|_, record| record.frame + KEEP_FRAMES > frame);
        kept_layers.retain(|_, record| record.frame + KEEP_FRAMES > frame);
    }

    /// Forget every record (after a resize, a lost device or a failed frame).
    pub(crate) fn clear(&mut self) {
        self.records = Records::Invalidated;
    }
}
