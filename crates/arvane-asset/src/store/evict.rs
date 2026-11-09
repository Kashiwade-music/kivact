//! Role: Eviction that keeps the asset cache within its budget.
//!
//! Keeping the cache within its budget: which item goes first, and
//! removing items with their pixel ids reported.
//!
//! Invariants:
//! - Nothing the current or the previous frame used is evicted (before the
//!   host's first frame no use is protected), nor
//!   anything in flight (a pending load, a reload, a check, a pending
//!   raster). When only such items are left, the cache stays over budget
//!   until they are no longer used: drawing what is shown wins over the
//!   budget, so a working set larger than the budget settles instead of
//!   evicting and reloading itself every frame.
//! - An unprotected item larger than a quarter of the budget goes before
//!   any smaller one: one huge image (a 20 MP photo is 80 MB of a 256 MiB
//!   budget) is kept while it is shown, not at the cost of everything
//!   else once it is not.
//! - Pixels go before metadata: a raster source loses its pixels but keeps
//!   its entry (natural size, failure, key); an entry is removed only once
//!   it holds nothing else.
//! - Every pixel id that leaves is reported once (`AssetStore::evicted`).
//! - At most `MAX_TINTS` tinted copies of one asset at one size are kept
//!   beyond those the current frame uses: a color animation makes a new
//!   tint every frame, and the old ones go as the new ones come.

use kivact_types::Asset;

use super::AssetStore;
use super::entry::{Residency, Stamp, State, Variant, VariantKey, frames_bytes};
use crate::pixels::AssetImage;

/// The most tinted copies of one asset at one size kept beyond those the
/// current frame uses.
pub(super) const MAX_TINTS: usize = 4;

/// An item `make_room` can evict.
enum Victim {
    /// A whole idle entry.
    Entry(Asset),
    /// A raster source's pixels.
    Source(Asset),
    /// A ready variant.
    Variant(Asset, VariantKey),
}

impl AssetStore {
    /// Whether an item used at `stamp` is protected from eviction: used in
    /// the current or the previous frame.
    pub(super) fn protected(&self, stamp: Stamp) -> bool {
        matches!(
            (self.frame.number(), stamp.frame),
            (Some(current), Some(used)) if used.get().saturating_add(1) >= current.get()
        )
    }

    /// Evicts least recently used items until `incoming` more bytes fit
    /// the budget, or nothing more may go (see the module invariants).
    pub(super) fn make_room(&mut self, incoming: usize) {
        while self.bytes + incoming > self.budget {
            let Some(victim) = self.oldest() else {
                return;
            };
            self.evict(victim);
        }
    }

    /// Release all eligible unused cache entries and variants under memory pressure.
    ///
    /// Current/previous-frame assets and in-flight work remain valid. Pixel
    /// evictions use [`Self::take_evicted`]. The normal budget is unchanged;
    /// retained sources needed by live entries and reloads are preserved.
    pub fn release_caches(&mut self) {
        while let Some(victim) = self.oldest() {
            self.evict(victim);
        }
        for entry in self.entries.values_mut() {
            entry.variants.shrink_to_fit();
        }
        self.entries.shrink_to_fit();
        self.failures.shrink_to_fit();
        self.evicted.shrink_to_fit();
    }

    fn evict(&mut self, victim: Victim) {
        match victim {
            Victim::Entry(asset) => self.remove(&asset),
            Victim::Source(asset) => {
                let Some(entry) = self.entries.get_mut(&asset) else {
                    return;
                };
                if let State::Raster { residency, .. } = &mut entry.state
                    && let Residency::Resident { frames, .. } = residency
                {
                    self.bytes -= frames_bytes(frames);
                    self.evicted.extend(frames.iter().map(AssetImage::id));
                    *residency = Residency::Evicted;
                }
            }
            Victim::Variant(asset, key) => {
                let removed = self
                    .entries
                    .get_mut(&asset)
                    .and_then(|e| e.variants.remove(&key));
                if let Some(Variant::Ready { pixels, .. }) = removed {
                    self.bytes -= pixels.bytes();
                    self.evicted.push(pixels.id());
                }
            }
        }
    }

    /// Makes room for one more tinted copy of `asset` from the raster
    /// `base` (its size and crop): evicts the least recently used copies of
    /// it not used in the current frame until fewer than `MAX_TINTS` are
    /// left (or only current ones are). Scans only the entry's variants.
    pub(super) fn trim_tints(&mut self, asset: &Asset, base: VariantKey) {
        let frame = self.frame.number();
        let Some(entry) = self.entries.get_mut(asset) else {
            return;
        };
        loop {
            let mut count = 0;
            let mut oldest: Option<(Stamp, VariantKey)> = None;
            for (key, variant) in &entry.variants {
                if let Variant::Ready { used, .. } = variant
                    && key.size == base.size
                    && key.crop == base.crop
                    && key.tint.is_some()
                {
                    count += 1;
                    let current = frame.is_some_and(|frame| used.frame == Some(frame));
                    if !current && oldest.is_none_or(|(o, _)| *used < o) {
                        oldest = Some((*used, *key));
                    }
                }
            }
            let Some((_, key)) = oldest.filter(|_| count >= MAX_TINTS) else {
                return;
            };
            if let Some(Variant::Ready { pixels, .. }) = entry.variants.remove(&key) {
                self.bytes -= pixels.bytes();
                self.evicted.push(pixels.id());
            }
        }
    }

    /// The item that may go first, if any: the least recently used of the
    /// large ones, else of all (see the module invariants).
    fn oldest(&self) -> Option<Victim> {
        let large = self.budget / super::asset_store::RETAINED_SHARE;
        let mut best: Option<((bool, Stamp), Victim)> = None;
        let mut consider = |stamp: Stamp, bytes: usize, victim: Victim| {
            let rank = (bytes <= large, stamp);
            if !self.protected(stamp) && best.as_ref().is_none_or(|(b, _)| rank < *b) {
                best = Some((rank, victim));
            }
        };
        for (asset, entry) in &self.entries {
            if entry.is_idle() {
                consider(entry.used, entry.bytes_held(), Victim::Entry(asset.clone()));
                continue;
            }
            if let State::Raster {
                residency: Residency::Resident { frames, drawn },
                ..
            } = &entry.state
            {
                consider(*drawn, frames_bytes(frames), Victim::Source(asset.clone()));
            }
            for (key, variant) in &entry.variants {
                if let Variant::Ready { used, pixels } = variant {
                    consider(*used, pixels.bytes(), Victim::Variant(asset.clone(), *key));
                }
            }
        }
        best.map(|(_, v)| v)
    }

    /// Drops `asset`'s entry, reporting its pixel ids.
    pub(super) fn remove(&mut self, asset: &Asset) {
        let Some(entry) = self.entries.remove(asset) else {
            return;
        };
        self.bytes -= entry.bytes_held();
        if let State::Raster {
            residency: Residency::Resident { frames, .. },
            ..
        } = &entry.state
        {
            self.evicted.extend(frames.iter().map(AssetImage::id));
        }
        for variant in entry.variants.values() {
            if let Variant::Ready { pixels, .. } = variant {
                self.evicted.push(pixels.id());
            }
        }
    }
}
