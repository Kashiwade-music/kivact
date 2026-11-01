//! Role: The asset store: one per host instance.
//! It loads images on its worker
//! thread (or inline), keeps decoded pixels in an LRU within a byte budget,
//! and answers what to draw.
//!
//! Invariants:
//! - One cache entry per [`Asset`] (by identity: a path, embedded bytes by
//!   address and length, shared bytes by allocation; the entry's key keeps
//!   a shared allocation alive so its address is not reused). Mounting the
//!   same asset any number of times loads it once. A relative path is
//!   keyed by its absolute form against the current directory at the call
//!   ([`key`]), so after the current directory changes it names another
//!   file and another entry (std's meaning of a relative path); the assets
//!   [`AssetStore::take_failures`] returns are these keys.
//! - Failures are cached like images. A failed file is loaded again only
//!   when its length or modification time changes, checked at most once
//!   per `RECHECK_INTERVAL` and on the worker (the check reads metadata,
//!   which blocks on an unreachable network path; an inline store checks
//!   in the call, still throttled). A URL failure is never checked again.
//!   A loaded file is not watched (automatic reload is out of scope).
//! - The budget counts decoded pixels (sources, SVG rasters, tinted
//!   variants), the parsed SVG trees (estimated), the encoded bytes entry
//!   keys keep alive and a fixed cost per entry (`entry::ENTRY_BYTES`).
//!   Recency is per frame: the host calls [`AssetStore::begin_frame`]
//!   before a pass that draws every image it shows, and what the current
//!   and the previous frame used is never evicted (`evict`). Pixels larger
//!   than the whole budget are not kept: a decoded image that large fails
//!   as `TooLarge`.
//! - Asking for a natural size never starts work: [`AssetStore::peek`]
//!   reads the cache only, and a raster source whose pixels were evicted
//!   keeps its natural size (its pixels are loaded again when drawn).
//! - Every evicted or replaced pixel id is reported once through
//!   [`AssetStore::take_evicted`]; every failure once through
//!   [`AssetStore::take_failures`].
//! - [`AssetStore::pixels`] never returns pixels of another asset. While a
//!   new SVG raster of an asset is pending, it returns that asset's most
//!   recent raster (scaled by the host), never an empty frame. A raster
//!   that lands is stamped with the frame its request was last made in, so
//!   a size no longer asked for is evictable at once (and dropped when the
//!   asset has another raster to fall back on).
//! - Results of jobs for entries evicted or replaced meanwhile are dropped
//!   (each entry has a generation).
//! - Thread: the store lives on its host's UI thread. Tints are computed
//!   on the calling thread (one pass over the pixels, cached per color);
//!   loads, decodes and rasters run on the worker in worker mode.

mod entry;
pub(crate) mod image_state;
use image_state::ImageState;
mod evict;
mod job;
mod worker;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ui_types::{Rgba, Size};

use ui_types::Asset;

use crate::error::AssetError;
use crate::fit::Region;
use crate::limits::ENTRY_BYTES;
use crate::limits::MAX_SVG_RASTER_EDGE;
use crate::pixels::{self, Bitmap, PixelSize, Pixels, Tint};
use crate::svg::Svg;
use entry::{Entry, Stamp, State, Variant, VariantKey};
use job::{Done, Job, Loaded};
use worker::Worker;

/// The default byte budget of a store's cache (256 MiB).
pub const DEFAULT_BUDGET: usize = 256 * 1024 * 1024;

/// One retained SVG raster takes at most this share of the budget
/// (`budget / RETAINED_SHARE`); a larger one is rasterized smaller.
const RETAINED_SHARE: usize = 4;

/// How often a failed file is checked for a change, at most.
const RECHECK_INTERVAL: Duration = Duration::from_secs(2);

/// Where jobs run.
enum Runner {
    Inline,
    Worker(Worker),
}

/// Loads, decodes and keeps images for one host instance (see the module
/// invariants).
pub struct AssetStore {
    runner: Runner,
    entries: HashMap<Asset, Entry>,
    /// The current frame (`begin_frame`); starts at 1.
    frame: u64,
    /// Ticks once per use; orders uses within a frame.
    tick: u64,
    next_id: u64,
    next_generation: u64,
    budget: usize,
    bytes: usize,
    failures: Vec<(Asset, AssetError)>,
    evicted: Vec<u64>,
    recheck_interval: Duration,
}

impl AssetStore {
    /// A store whose jobs run on its own worker thread; finished jobs wake
    /// the host by calling `waker` (on the worker thread), and the host
    /// calls [`AssetStore::poll`]. When the system cannot start a thread,
    /// jobs run inline instead.
    pub fn new(waker: impl Fn() + Send + 'static) -> AssetStore {
        let runner = Worker::spawn(waker).map_or(Runner::Inline, Runner::Worker);
        AssetStore::with_runner(runner)
    }

    /// A store that runs every job synchronously inside the call that needs
    /// it, so results are deterministic (the test driver's mode).
    pub fn inline() -> AssetStore {
        AssetStore::with_runner(Runner::Inline)
    }

    fn with_runner(runner: Runner) -> AssetStore {
        AssetStore {
            runner,
            entries: HashMap::new(),
            frame: 1,
            tick: 0,
            next_id: 1,
            next_generation: 1,
            budget: DEFAULT_BUDGET,
            bytes: 0,
            failures: Vec::new(),
            evicted: Vec::new(),
            recheck_interval: RECHECK_INTERVAL,
        }
    }

    /// Starts a frame: the host calls it before a pass that draws every
    /// image it shows (each frame for a host that rebuilds its frame, each
    /// redraw of all images for one that caches drawings). What the current
    /// and the previous frame used stays in the cache, even over budget.
    pub fn begin_frame(&mut self) {
        self.frame += 1;
    }

    /// What the cache knows about `asset`, starting nothing and using
    /// nothing (`None`: not in the cache). For lookups that must not load,
    /// such as a host's sweep over its image nodes after `poll`.
    pub fn peek(&self, asset: &Asset) -> Option<ImageState> {
        self.entries.get(key(asset).as_ref()).map(Entry::state)
    }

    /// The state of the image `asset`, starting its load when it is not in
    /// the cache (or checking a failed file for a change, throttled).
    pub fn image(&mut self, asset: &Asset) -> ImageState {
        let asset = key(asset);
        let asset = asset.as_ref();
        let now = self.stamp();
        if let Some(entry) = self.entries.get_mut(asset) {
            entry.used = now;
            if entry.check_due(asset) {
                self.start_check(asset);
            }
        } else {
            self.start_load(asset, now);
        }
        self.entries
            .get(asset)
            .map_or(ImageState::Pending, Entry::state)
    }

    /// The pixels to draw for `asset`: a raster image's own pixels, or an
    /// SVG rasterized at `device` (within the raster limits), tinted with
    /// `tint` when given. `None` while nothing of this asset can be drawn
    /// (pending, failed, empty size, evicted pixels loading again).
    ///
    /// While the SVG raster at `device` is pending, the asset's most recent
    /// whole raster is returned (with the same tint when cached, else
    /// tinted now); the host scales it to the rect.
    pub fn pixels(
        &mut self,
        asset: &Asset,
        device: PixelSize,
        tint: Option<Rgba>,
    ) -> Option<Pixels> {
        self.draw(asset, device, Region::WHOLE, tint, true)
            .map(|(pixels, _)| pixels)
    }

    /// The pixels to draw for the part `visible` of `asset` drawn at
    /// `device` pixels (an overflowing image clipped to its box,
    /// [`FitRect::visible`](crate::FitRect::visible)), and the part of the
    /// image they cover: the host draws them at that part of its rect
    /// ([`Region::place`]). An SVG is rasterized for the visible part only,
    /// at device resolution; a raster image gives all its pixels. While
    /// the raster is pending, the asset's most recent raster of any part
    /// is returned with its own region.
    ///
    /// The raster of an SVG is at most `MAX_SVG_RASTER_EDGE` per edge and a
    /// quarter of the budget; past that it is made smaller and drawn scaled
    /// up, so one image can neither fail to draw nor take the whole cache.
    pub fn pixels_in(
        &mut self,
        asset: &Asset,
        device: PixelSize,
        visible: Region,
        tint: Option<Rgba>,
    ) -> Option<(Pixels, Region)> {
        self.draw(asset, device, visible, tint, false)
    }

    /// `pixels` and `pixels_in`; `whole` limits fallbacks to whole rasters.
    fn draw(
        &mut self,
        asset: &Asset,
        device: PixelSize,
        visible: Region,
        tint: Option<Rgba>,
        whole: bool,
    ) -> Option<(Pixels, Region)> {
        let asset = key(asset);
        let asset = asset.as_ref();
        if !matches!(self.image(asset), ImageState::Ready { .. }) {
            return None;
        }
        let now = self.stamp();
        let tint = tint.map(pixels::tint_key);
        let max_bytes = self.budget / RETAINED_SHARE;
        let entry = self.entries.get_mut(asset)?;
        match &mut entry.state {
            State::Raster {
                pixels,
                drawn,
                reloading,
                ..
            } => {
                *drawn = now;
                if pixels.is_none() && !*reloading {
                    *reloading = true;
                    self.start_reload(asset);
                }
                let source = match &self.entries.get(asset)?.state {
                    State::Raster {
                        pixels: Some(p), ..
                    } => p.clone(),
                    _ => return None,
                };
                let base = VariantKey {
                    size: None,
                    crop: None,
                    tint: None,
                };
                self.tinted(asset, base, source, tint)
                    .map(|p| (p, Region::WHOLE))
            }
            State::Vector { svg, .. } => {
                let (size, crop) =
                    pixels::raster_plan(device, visible, MAX_SVG_RASTER_EDGE, max_bytes)?;
                let svg = Arc::clone(svg);
                let key = VariantKey {
                    size: Some(size),
                    crop,
                    tint: None,
                };
                match entry.variants.get_mut(&key) {
                    None => self.start_raster(asset, svg, key, now),
                    Some(Variant::Pending { requested }) => *requested = now,
                    Some(Variant::Ready { .. } | Variant::Rejected) => {}
                }
                match self.ready_variant(asset, key) {
                    Some(base) => self
                        .tinted(asset, key, base, tint)
                        .map(|p| (p, key.region())),
                    None => self.fallback(asset, tint, whole),
                }
            }
            State::Pending | State::Failed(_) => None,
        }
    }

    /// Applies finished jobs; true when any image state or raster changed
    /// (the host lays out and redraws then). Always false for an inline
    /// store, whose jobs finish inside the calls that start them.
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        loop {
            let done = match &self.runner {
                Runner::Worker(worker) => worker.try_recv(),
                Runner::Inline => None,
            };
            let Some(done) = done else {
                return changed;
            };
            changed |= self.apply(done);
        }
    }

    /// Failures since the last call, each once: the asset and its error.
    pub fn take_failures(&mut self) -> Vec<(Asset, AssetError)> {
        std::mem::take(&mut self.failures)
    }

    /// Ids of pixels evicted or replaced since the last call; a host frees
    /// the device textures it made from them.
    pub fn take_evicted(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.evicted)
    }

    /// A stamp for a use now.
    fn stamp(&mut self) -> Stamp {
        self.tick += 1;
        Stamp {
            frame: self.frame,
            tick: self.tick,
        }
    }

    fn submit(&mut self, job: Job) {
        match &self.runner {
            Runner::Worker(worker) => worker.submit(job),
            Runner::Inline => {
                let done = worker::run_on_job_stack(job);
                self.apply(done);
            }
        }
    }

    fn start_load(&mut self, asset: &Asset, now: Stamp) {
        let retained = match asset {
            Asset::Shared(b) => b.len(),
            Asset::Path(_) | Asset::Static(_) => 0,
        };
        self.make_room(ENTRY_BYTES + retained);
        let generation = self.next_generation;
        self.next_generation += 1;
        self.bytes += ENTRY_BYTES + retained;
        self.entries
            .insert(asset.clone(), Entry::pending(generation, retained, now));
        self.submit(Job::Load {
            asset: asset.clone(),
            generation,
            budget: self.budget,
            retained: ENTRY_BYTES + retained,
        });
    }

    /// Loads the evicted pixels of the raster source `asset` again (its
    /// state stays ready meanwhile).
    fn start_reload(&mut self, asset: &Asset) {
        let Some(entry) = self.entries.get(asset) else {
            return;
        };
        let job = Job::Load {
            asset: asset.clone(),
            generation: entry.generation,
            budget: self.budget,
            retained: ENTRY_BYTES + entry.retained,
        };
        self.submit(job);
    }

    /// Checks the failed file `asset` for a change (on the worker).
    fn start_check(&mut self, asset: &Asset) {
        let Some(entry) = self.entries.get_mut(asset) else {
            return;
        };
        entry.checking = true;
        let job = Job::Recheck {
            asset: asset.clone(),
            generation: entry.generation,
            version: entry.version,
            budget: self.budget,
        };
        self.submit(job);
    }

    /// Rasterizes `svg` for the variant `key` (its size and crop).
    fn start_raster(&mut self, asset: &Asset, svg: Arc<Svg>, key: VariantKey, now: Stamp) {
        let (Some(entry), Some(size)) = (self.entries.get_mut(asset), key.size) else {
            return;
        };
        entry
            .variants
            .insert(key, Variant::Pending { requested: now });
        let generation = entry.generation;
        self.submit(Job::Raster {
            asset: asset.clone(),
            generation,
            svg,
            size,
            crop: key.crop,
        });
    }

    /// Applies a finished job; whether anything a host shows changed.
    fn apply(&mut self, done: Done) -> bool {
        match done {
            Done::Load {
                asset,
                generation,
                version,
                result,
            } => {
                if self.entries.get(&asset).map(|e| e.generation) != Some(generation) {
                    return false;
                }
                let state = match result {
                    Ok(Loaded::Raster(bitmap)) => self.admit_source(&asset, bitmap),
                    Ok(Loaded::Vector(svg)) => {
                        self.make_room(svg.bytes());
                        let natural = svg.natural();
                        State::Vector { svg, natural }
                    }
                    Err(e) => State::Failed(e),
                };
                if let State::Failed(e) = &state {
                    self.failures.push((asset.clone(), e.clone()));
                }
                let next_check = Instant::now() + self.recheck_interval;
                let Some(entry) = self.entries.get_mut(&asset) else {
                    return false;
                };
                self.bytes -= entry.state.bytes();
                self.bytes += state.bytes();
                entry.state = state;
                entry.version = version;
                entry.checking = false;
                entry.next_check = Some(next_check);
                true
            }
            Done::Unchanged { asset, generation } => {
                let next_check = Instant::now() + self.recheck_interval;
                if let Some(entry) = self
                    .entries
                    .get_mut(&asset)
                    .filter(|e| e.generation == generation)
                {
                    entry.checking = false;
                    entry.next_check = Some(next_check);
                }
                false
            }
            Done::Raster {
                asset,
                generation,
                size,
                crop,
                bitmap,
            } => {
                if self.entries.get(&asset).map(|e| e.generation) != Some(generation) {
                    return false;
                }
                let key = VariantKey {
                    size: Some(size),
                    crop,
                    tint: None,
                };
                self.land_raster(&asset, key, bitmap)
            }
        }
    }

    /// The state for a decoded raster source (first load or reload): ready
    /// when it fits the budget (making room for it), else a `TooLarge`
    /// failure. The pixels count as drawn now: they were asked for.
    fn admit_source(&mut self, asset: &Asset, bitmap: Bitmap) -> State {
        let retained = self.entries.get(asset).map_or(0, |e| e.retained);
        if ENTRY_BYTES + retained + bitmap.bytes() > self.budget {
            return State::Failed(job::over_budget(
                asset,
                bitmap.width,
                bitmap.height,
                bitmap.bytes() as u64,
                self.budget,
            ));
        }
        self.make_room(bitmap.bytes());
        let natural = Size {
            width: bitmap.width as f32,
            height: bitmap.height as f32,
        };
        let drawn = self.stamp();
        State::Raster {
            natural,
            pixels: Some(self.new_pixels(bitmap)),
            drawn,
            reloading: false,
        }
    }

    /// Stores an SVG raster that landed, stamped with the last request for
    /// it. A raster no longer asked for (not in the current or the previous
    /// frame) is dropped when the asset has another ready raster to fall
    /// back on. Returns whether it was stored.
    fn land_raster(&mut self, asset: &Asset, key: VariantKey, bitmap: Bitmap) -> bool {
        let Some(entry) = self.entries.get(asset) else {
            return false;
        };
        let requested = match entry.variants.get(&key) {
            Some(Variant::Pending { requested }) => Some(*requested),
            _ => None,
        };
        let fallback = entry
            .variants
            .iter()
            .any(|(k, v)| *k != key && k.tint.is_none() && matches!(v, Variant::Ready { .. }));
        let requested = requested.unwrap_or_else(|| self.stamp());
        if !self.protected(requested) && fallback {
            if let Some(entry) = self.entries.get_mut(asset) {
                entry.variants.remove(&key);
            }
            return false;
        }
        self.insert_variant(asset, key, bitmap, requested).is_some()
    }

    /// Stores a ready variant used at `used`, or marks it rejected when
    /// larger than the whole budget. Returns its pixels when stored.
    fn insert_variant(
        &mut self,
        asset: &Asset,
        key: VariantKey,
        bitmap: Bitmap,
        used: Stamp,
    ) -> Option<Pixels> {
        if bitmap.bytes() > self.budget {
            if let Some(entry) = self.entries.get_mut(asset) {
                entry.variants.insert(key, Variant::Rejected);
            }
            return None;
        }
        self.make_room(bitmap.bytes());
        let pixels = self.new_pixels(bitmap);
        let entry = self.entries.get_mut(asset)?;
        self.bytes += pixels.bytes();
        let old = entry.variants.insert(
            key,
            Variant::Ready {
                pixels: pixels.clone(),
                used,
            },
        );
        if let Some(Variant::Ready { pixels: old, .. }) = old {
            self.bytes -= old.bytes();
            self.evicted.push(old.id());
        }
        Some(pixels)
    }

    fn new_pixels(&mut self, bitmap: Bitmap) -> Pixels {
        let id = self.next_id;
        self.next_id += 1;
        Pixels::new(id, bitmap)
    }

    /// The ready variant `key` of `asset`, marked used now.
    fn ready_variant(&mut self, asset: &Asset, key: VariantKey) -> Option<Pixels> {
        let now = self.stamp();
        match self.entries.get_mut(asset)?.variants.get_mut(&key)? {
            Variant::Ready { pixels, used } => {
                *used = now;
                Some(pixels.clone())
            }
            Variant::Pending { .. } | Variant::Rejected => None,
        }
    }

    /// `base` (the source or an SVG raster, the variant `from`) in `tint`,
    /// computing and caching the tinted copy when needed.
    fn tinted(
        &mut self,
        asset: &Asset,
        from: VariantKey,
        base: Pixels,
        tint: Option<Tint>,
    ) -> Option<Pixels> {
        let Some(t) = tint else {
            return Some(base);
        };
        let key = VariantKey {
            tint: Some(t),
            ..from
        };
        if let Some(pixels) = self.ready_variant(asset, key) {
            return Some(pixels);
        }
        if matches!(
            self.entries.get(asset)?.variants.get(&key),
            Some(Variant::Rejected)
        ) {
            return None;
        }
        let bitmap = pixels::tint(base.width(), base.height(), base.rgba(), t);
        self.trim_tints(asset, from);
        let now = self.stamp();
        self.insert_variant(asset, key, bitmap, now)
    }

    /// The most recent raster of a vector `asset` in `tint` (whole ones only
    /// when `whole`), and the part it covers: a cached one with that tint,
    /// else the most recent untinted raster, tinted now.
    fn fallback(
        &mut self,
        asset: &Asset,
        tint: Option<Tint>,
        whole: bool,
    ) -> Option<(Pixels, Region)> {
        let entry = self.entries.get(asset)?;
        let newest = |want: Option<Tint>| {
            entry
                .variants
                .iter()
                .filter_map(|(k, v)| match v {
                    Variant::Ready { used, .. }
                        if k.tint == want && (!whole || k.crop.is_none()) =>
                    {
                        Some((*used, *k))
                    }
                    _ => None,
                })
                .max_by_key(|(used, _)| *used)
                .map(|(_, k)| k)
        };
        if let Some(key) = newest(tint) {
            return self.ready_variant(asset, key).map(|p| (p, key.region()));
        }
        let key = newest(None)?;
        let base = self.ready_variant(asset, key)?;
        self.tinted(asset, key, base, tint)
            .map(|p| (p, key.region()))
    }
}

/// The cache key of `asset`: a relative path made absolute against the
/// current directory now (a URL is left as it is); any other asset itself.
pub(crate) fn key(asset: &Asset) -> Cow<'_, Asset> {
    match asset {
        Asset::Path(p) if p.is_relative() && !crate::source::is_url(p) => {
            match std::path::absolute(p) {
                Ok(abs) => Cow::Owned(Asset::Path(abs)),
                Err(_) => Cow::Borrowed(asset),
            }
        }
        _ => Cow::Borrowed(asset),
    }
}
