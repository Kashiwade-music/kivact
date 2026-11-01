//! Role: Cache entry of the asset store: its state, derived variants and stamps.
//!
//! A cache entry and what it holds: the asset's state (pending, a raster
//! source, a parsed SVG, a failure), its derived variants (SVG rasters and
//! tinted copies), the stamps that order them for eviction, and the bytes
//! they count in the budget.
//!
//! Invariants:
//! - [`Entry::bytes_held`] is exactly what the store's byte count holds for
//!   the entry: `ENTRY_BYTES`, the encoded bytes the key keeps alive, the
//!   state's bytes (source pixels or the parsed tree's estimate) and every
//!   ready variant.
//! - A raster source whose pixels were evicted keeps its natural size, so
//!   layout never waits for a reload; the pixels come back when drawn.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use kivact_types::Size;

use kivact_types::Asset;

use crate::error::{AssetError, AssetErrorKind};
use crate::pixels::{Crop, PixelSize, Pixels, Tint};
use crate::read::FileVersion;
use crate::svg::Svg;

use super::image_state::ImageState;
use crate::limits::ENTRY_BYTES;

/// When an item was last used: the frame (see `AssetStore::begin_frame`)
/// and a tick within it. Orders the LRU; the frame also protects what the
/// current and the previous frame used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Stamp {
    pub(crate) frame: u64,
    pub(crate) tick: u64,
}

pub(crate) struct Entry {
    pub(crate) generation: u64,
    pub(crate) state: State,
    /// The file version the state was loaded from (files only).
    pub(crate) version: FileVersion,
    /// A failed file: when it may be checked for a change next, and
    /// whether a check is in flight.
    pub(crate) next_check: Option<Instant>,
    pub(crate) checking: bool,
    /// Encoded bytes the key keeps alive.
    pub(crate) retained: usize,
    /// The last `image` or `pixels` call for the asset.
    pub(crate) used: Stamp,
    pub(crate) variants: HashMap<VariantKey, Variant>,
}

pub(crate) enum State {
    /// The first load is in flight.
    Pending,
    /// A raster image: its natural size, and its pixels unless evicted
    /// (`drawn` is their last use; `reloading` while a load brings them
    /// back).
    Raster {
        natural: Size,
        pixels: Option<Pixels>,
        drawn: Stamp,
        reloading: bool,
    },
    Vector {
        svg: Arc<Svg>,
        natural: Size,
    },
    Failed(AssetError),
}

/// A derived raster: an SVG at a device size (or the part `crop` of it),
/// and/or a tinted copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct VariantKey {
    /// The device size (SVG rasters only).
    pub(crate) size: Option<PixelSize>,
    /// The part of that size rasterized (SVG rasters of a visible part).
    pub(crate) crop: Option<Crop>,
    pub(crate) tint: Option<Tint>,
}

impl VariantKey {
    /// The part of the image this variant covers.
    pub(crate) fn region(&self) -> crate::fit::Region {
        match self.size {
            Some(size) => Crop::region(self.crop, size),
            None => crate::fit::Region::WHOLE,
        }
    }
}

pub(crate) enum Variant {
    /// A raster in flight; `requested` is the last call that asked for it.
    Pending {
        requested: Stamp,
    },
    Ready {
        pixels: Pixels,
        used: Stamp,
    },
    /// Larger than the whole budget; not asked for again.
    Rejected,
}

impl State {
    /// Bytes the state holds in the budget.
    pub(crate) fn bytes(&self) -> usize {
        match self {
            State::Raster { pixels, .. } => pixels.as_ref().map_or(0, Pixels::bytes),
            State::Vector { svg, .. } => svg.bytes(),
            State::Pending | State::Failed(_) => 0,
        }
    }
}

impl Entry {
    /// A pending entry (its first load in flight).
    pub(crate) fn pending(generation: u64, retained: usize, used: Stamp) -> Entry {
        Entry {
            generation,
            state: State::Pending,
            version: None,
            next_check: None,
            checking: false,
            retained,
            used,
            variants: HashMap::new(),
        }
    }

    /// Whether this is a failed file due for a check for a change (never a
    /// URL, never while a check is in flight).
    pub(crate) fn check_due(&self, asset: &Asset) -> bool {
        match (&self.state, asset) {
            (State::Failed(e), Asset::Path(_)) => {
                e.kind() != AssetErrorKind::Url
                    && !self.checking
                    && self.next_check.is_none_or(|t| Instant::now() >= t)
            }
            _ => false,
        }
    }

    pub(crate) fn state(&self) -> ImageState {
        match &self.state {
            State::Pending => ImageState::Pending,
            State::Raster { natural, .. } | State::Vector { natural, .. } => {
                ImageState::Ready { natural: *natural }
            }
            State::Failed(e) => ImageState::Failed(e.clone()),
        }
    }

    /// Whether the entry holds only its metadata (no pixels, no variants,
    /// nothing in flight), so removing it loses no work but the load.
    pub(crate) fn is_idle(&self) -> bool {
        let busy = match &self.state {
            State::Pending => true,
            State::Raster {
                pixels, reloading, ..
            } => pixels.is_some() || *reloading,
            State::Vector { .. } | State::Failed(_) => false,
        };
        !busy
            && !self.checking
            && self
                .variants
                .values()
                .all(|v| matches!(v, Variant::Rejected))
    }

    /// Bytes this entry holds in the budget (see the module invariants).
    pub(crate) fn bytes_held(&self) -> usize {
        let variants: usize = self
            .variants
            .values()
            .map(|v| match v {
                Variant::Ready { pixels, .. } => pixels.bytes(),
                Variant::Pending { .. } | Variant::Rejected => 0,
            })
            .sum();
        ENTRY_BYTES + self.retained + self.state.bytes() + variants
    }
}
