//! Role: AssetStore: schedules asset jobs and answers images and pixels from its cache.

use super::entry::{Entry, Recheck, Residency, Stamp, State, Variant, VariantKey};
use super::image_state::{AnimationInfo, ImageState};
use super::job::{Done, Job, Loaded};
use super::worker::Worker;
use crate::error::AssetError;
use crate::fit::Region;
use crate::limits::ENTRY_BYTES;
use crate::limits::MAX_SVG_RASTER_EDGE;
use crate::pixels::{self, AssetImage, PixelSize, Tint};
use crate::raster::Decoded;
use crate::svg::{FontDb, Svg};
use kivact_types::Asset;
use kivact_types::Pixels;
use kivact_types::{Rgba, Size};
use resvg::usvg;
use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The default byte budget of a store's cache (256 MiB).
pub const DEFAULT_BUDGET: usize = 256 * 1024 * 1024;

/// One retained SVG raster takes at most this share of the budget
/// (`budget / RETAINED_SHARE`); a larger one is rasterized smaller.
pub(super) const RETAINED_SHARE: usize = 4;

/// How often a failed file is checked for a change, at most.
const RECHECK_INTERVAL: Duration = Duration::from_secs(2);

/// Where jobs run.
pub(super) enum Runner {
    Inline,
    Worker(Worker),
}

/// Loads, decodes and keeps images for one host instance (see the module
/// invariants).
pub struct AssetStore {
    pub(super) runner: Runner,
    /// The fonts SVG `<text>` is shaped with: the system fonts and the
    /// registered ones (`register_font`).
    pub(super) fonts: FontDb,
    pub(super) entries: HashMap<Asset, Entry>,
    /// The host's current frame (`begin_frame`, counted from 1); `None`
    /// before the first: nothing is protected.
    pub(super) frame: Option<NonZeroU64>,
    /// Ticks once per use; orders uses within a frame.
    pub(super) tick: u64,
    pub(super) next_id: u64,
    pub(super) next_generation: u64,
    pub(super) budget: usize,
    pub(super) bytes: usize,
    pub(super) failures: Vec<(Asset, AssetError)>,
    pub(super) evicted: Vec<u64>,
    pub(super) recheck_interval: Duration,
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
        let mut fonts = usvg::fontdb::Database::new();
        fonts.load_system_fonts();
        AssetStore {
            runner,
            fonts: Arc::new(fonts),
            entries: HashMap::new(),
            frame: None,
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
    /// Until the first call nothing is protected: the cache is a plain LRU
    /// within its budget.
    pub fn begin_frame(&mut self) {
        self.frame = Some(match self.frame {
            Some(frame) => frame.checked_add(1).expect("invariant: frames fit u64"),
            None => NonZeroU64::MIN,
        });
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
            if entry.check_due() {
                self.start_check(asset);
            }
        } else {
            self.start_load(asset, now);
        }
        self.entries
            .get(asset)
            .map_or(ImageState::Pending, Entry::state)
    }

    /// The timing of the animated image `asset` (each frame's delay and how
    /// often it repeats), starting its load when it is not in the cache.
    /// `None` while it is not ready, or when it is not an animation.
    pub fn animation(&mut self, asset: &Asset) -> Option<AnimationInfo> {
        if !matches!(self.image(asset), ImageState::Ready { .. }) {
            return None;
        }
        let asset = key(asset);
        match &self.entries.get(asset.as_ref())?.state {
            State::Raster { animation, .. } => animation.clone(),
            _ => None,
        }
    }

    /// Registers the font `bytes` for SVG `<text>`. Every SVG that is pending
    /// or parsed is parsed again with the new fonts (its rasters are dropped,
    /// and the results of the parses of the old fonts are dropped when they
    /// land).
    pub fn register_font(&mut self, bytes: Arc<[u8]>) {
        Arc::make_mut(&mut self.fonts).load_font_data(bytes.to_vec());
        let stale: Vec<Asset> = self
            .entries
            .iter()
            .filter(|(_, entry)| matches!(entry.state, State::Pending | State::Vector { .. }))
            .map(|(asset, _)| asset.clone())
            .collect();
        for asset in &stale {
            self.reparse(asset);
        }
    }

    /// Drops the rasters and the state of `asset`'s entry and loads it again
    /// under a new generation (so an older job's result is dropped).
    fn reparse(&mut self, asset: &Asset) {
        let Some(entry) = self.entries.get_mut(asset) else {
            return;
        };
        self.bytes -= entry.bytes_held();
        for variant in entry.variants.values() {
            if let Variant::Ready { pixels, .. } = variant {
                self.evicted.push(pixels.id());
            }
        }
        entry.variants.clear();
        entry.state = State::Pending;
        entry.generation = self.next_generation;
        self.next_generation += 1;
        self.bytes += entry.bytes_held();
        self.start_reload(asset);
    }

    /// The pixels to draw for `asset`: a raster image's own pixels, or an
    /// SVG rasterized at `device` (within the raster limits), tinted with
    /// `tint` when given. `None` while nothing of this asset can be drawn
    /// (pending, failed, empty size, evicted pixels loading again).
    ///
    /// `frame` is the frame of an animation (0 for a still image and an
    /// SVG); a frame past the last one is the last. While the SVG raster at
    /// `device` is pending, the asset's most recent whole raster is returned
    /// (with the same tint when cached, else tinted now); the host scales it
    /// to the rect.
    pub fn pixels(
        &mut self,
        asset: &Asset,
        device: PixelSize,
        tint: Option<Rgba>,
        frame: u32,
    ) -> Option<AssetImage> {
        self.draw(asset, device, Region::WHOLE, tint, true, frame)
            .map(|(pixels, _)| pixels)
    }

    /// The pixels to draw for the part `visible` of `asset` drawn at
    /// `device` pixels (an overflowing image clipped to its box,
    /// [`FitRect::visible`](crate::FitRect::visible)), and the part of the
    /// image they cover: the host draws them at that part of its rect
    /// ([`Region::place`]). An SVG is rasterized for the visible part only,
    /// at device resolution; a raster image gives all its pixels. While
    /// the raster is pending, the asset's most recent raster of any part
    /// is returned with its own region. `frame` is as for `pixels`.
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
        frame: u32,
    ) -> Option<(AssetImage, Region)> {
        self.draw(asset, device, visible, tint, false, frame)
    }

    /// `pixels` and `pixels_in`; `whole` limits fallbacks to whole rasters.
    fn draw(
        &mut self,
        asset: &Asset,
        device: PixelSize,
        visible: Region,
        tint: Option<Rgba>,
        whole: bool,
        frame: u32,
    ) -> Option<(AssetImage, Region)> {
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
            State::Raster { residency, .. } => {
                match residency {
                    Residency::Resident { drawn, .. } => *drawn = now,
                    Residency::Evicted => {
                        *residency = Residency::Reloading;
                        self.start_reload(asset);
                    }
                    Residency::Reloading => {}
                }
                let (index, source) = self.resident_frame(asset, frame)?;
                let base = VariantKey {
                    size: None,
                    crop: None,
                    tint: None,
                    frame: index,
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
                    frame: 0,
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
            State::Pending | State::Failed { .. } => None,
        }
    }

    /// Applies finished jobs; true when any image state or raster changed
    /// (the host lays out and redraws then). Always false for an inline
    /// store, whose jobs finish inside the calls that start them.
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        loop {
            let done = match &mut self.runner {
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
        match &mut self.runner {
            Runner::Worker(worker) => worker.submit(job),
            Runner::Inline => {
                let done = super::worker::run_on_job_stack(job);
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
            fonts: Arc::clone(&self.fonts),
        });
    }

    /// Loads the evicted pixels of the raster source `asset` again (its
    /// state stays ready meanwhile); also loads a reparsed SVG.
    fn start_reload(&mut self, asset: &Asset) {
        let Some(entry) = self.entries.get(asset) else {
            return;
        };
        let job = Job::Load {
            asset: asset.clone(),
            generation: entry.generation,
            budget: self.budget,
            retained: ENTRY_BYTES + entry.retained,
            fonts: Arc::clone(&self.fonts),
        };
        self.submit(job);
    }

    /// Checks the failed file `asset` for a change (on the worker).
    fn start_check(&mut self, asset: &Asset) {
        let Some(entry) = self.entries.get_mut(asset) else {
            return;
        };
        let State::Failed { recheck, .. } = &mut entry.state else {
            return;
        };
        let Recheck::Due { version, .. } = *recheck else {
            return;
        };
        *recheck = Recheck::Checking { version };
        let job = Job::Recheck {
            asset: asset.clone(),
            generation: entry.generation,
            version,
            budget: self.budget,
            fonts: Arc::clone(&self.fonts),
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
                let loaded = match result {
                    Ok(Loaded::Raster(Decoded::Still(bitmap))) => {
                        self.admit_frames(&asset, vec![bitmap], None)
                    }
                    Ok(Loaded::Raster(Decoded::Animated(animation))) => {
                        let (frames, delays): (Vec<Pixels>, Vec<u32>) =
                            animation.frames.into_iter().unzip();
                        let timing = AnimationInfo {
                            delays_ms: delays.into(),
                            repeat: animation.repeat,
                        };
                        self.admit_frames(&asset, frames, Some(timing))
                    }
                    Ok(Loaded::Vector(svg)) => {
                        self.make_room(svg.bytes());
                        let natural = svg.natural();
                        Ok(State::Vector { svg, natural })
                    }
                    Err(error) => Err(error),
                };
                let state = match loaded {
                    Ok(state) => state,
                    Err(error) => {
                        self.failures.push((asset.clone(), error.clone()));
                        let at = Instant::now() + self.recheck_interval;
                        State::Failed {
                            recheck: Recheck::after(&asset, &error, version, at),
                            error,
                        }
                    }
                };
                let Some(entry) = self.entries.get_mut(&asset) else {
                    return false;
                };
                self.bytes -= entry.state.bytes();
                self.bytes += state.bytes();
                entry.state = state;
                true
            }
            Done::Unchanged { asset, generation } => {
                let at = Instant::now() + self.recheck_interval;
                if let Some(entry) = self
                    .entries
                    .get_mut(&asset)
                    .filter(|e| e.generation == generation)
                    && let State::Failed { recheck, .. } = &mut entry.state
                    && let Recheck::Checking { version } = *recheck
                {
                    *recheck = Recheck::Due { at, version };
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
                    frame: 0,
                };
                self.land_raster(&asset, key, bitmap)
            }
        }
    }

    /// The state for the decoded frames of a raster source (first load or
    /// reload; one frame for a still image, `timing` for an animation): ready
    /// when they fit the budget (making room for them), else a `TooLarge`
    /// failure. The frames count as drawn now: they were asked for.
    fn admit_frames(
        &mut self,
        asset: &Asset,
        frames: Vec<Pixels>,
        timing: Option<AnimationInfo>,
    ) -> Result<State, AssetError> {
        let retained = self.entries.get(asset).map_or(0, |e| e.retained);
        let pixels: usize = frames.iter().map(|f| f.rgba.len()).sum();
        let timing_bytes = timing
            .as_ref()
            .map_or(0, |t| t.delays_ms.len() * std::mem::size_of::<u32>());
        let first = frames
            .first()
            .expect("invariant: a decoded raster source has a frame");
        if ENTRY_BYTES + retained + pixels + timing_bytes > self.budget {
            return Err(super::job::over_budget(
                asset,
                first.width,
                first.height,
                pixels as u64,
                self.budget,
            ));
        }
        self.make_room(pixels + timing_bytes);
        let natural = Size {
            width: f64::from(first.width),
            height: f64::from(first.height),
        };
        let drawn = self.stamp();
        let frames = frames
            .into_iter()
            .map(|bitmap| self.new_pixels(bitmap))
            .collect();
        Ok(State::Raster {
            natural,
            residency: Residency::Resident { frames, drawn },
            animation: timing,
        })
    }

    /// Stores an SVG raster that landed, stamped with the last request for
    /// it. A raster no longer asked for (not in the current or the previous
    /// frame) is dropped when the asset has another ready raster to fall
    /// back on. Returns whether it was stored.
    fn land_raster(&mut self, asset: &Asset, key: VariantKey, bitmap: Pixels) -> bool {
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
        bitmap: Pixels,
        used: Stamp,
    ) -> Option<AssetImage> {
        if bitmap.rgba.len() > self.budget {
            if let Some(entry) = self.entries.get_mut(asset) {
                entry.variants.insert(key, Variant::Rejected);
            }
            return None;
        }
        self.make_room(bitmap.rgba.len());
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

    fn new_pixels(&mut self, bitmap: Pixels) -> AssetImage {
        let id = self.next_id;
        self.next_id += 1;
        AssetImage::new(id, bitmap)
    }

    /// Frame `frame` of the resident raster source `asset` and its index
    /// (the last frame when `frame` is past the end); `None` when its
    /// pixels are not resident.
    fn resident_frame(&self, asset: &Asset, frame: u32) -> Option<(u32, AssetImage)> {
        let State::Raster {
            residency: Residency::Resident { frames, .. },
            ..
        } = &self.entries.get(asset)?.state
        else {
            return None;
        };
        let index = (frame as usize).min(frames.len().checked_sub(1)?);
        Some((u32::try_from(index).ok()?, frames[index].clone()))
    }

    /// The ready variant `key` of `asset`, marked used now.
    fn ready_variant(&mut self, asset: &Asset, key: VariantKey) -> Option<AssetImage> {
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
        base: AssetImage,
        tint: Option<Tint>,
    ) -> Option<AssetImage> {
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
    ) -> Option<(AssetImage, Region)> {
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
pub(super) fn key(asset: &Asset) -> Cow<'_, Asset> {
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
