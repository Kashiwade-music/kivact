//! Role: CpuCompositor: runs a planned frame on the calling thread.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use arvane_render::{CpuRaster, RenderError};
use kivact_types::gpu::ExternalFrame;
use kivact_types::ids::DrawableId;
use kivact_types::pixel::{BlendMode, Pixels};
use kivact_types::render::{
    Composite, Composition, CpuFrame, ExportFrame, ExportMode, ExportedFrame, FrameReport,
    FrameStats, LayerKey, PixelRect, RenderScene,
};

use super::draw::{self, Chunks};
use super::filters;
use super::mask;
use super::surface::Pixmap;
use crate::ComposeError;
use crate::cache::{CacheIndex, ChunkKey, ChunkRecord, LayerRecord};
use crate::plan::{
    self, ChunkTask, Damage, DrawOp, Op, Redraw, Sampling, Scissor, Source, TargetKind,
};

/// The output of one window frame.
pub struct CpuOutput<'a> {
    /// The window frame: width × height premultiplied RGBA8, kept across frames.
    pub rgba: &'a [u8],
    /// Width in px.
    pub width: u32,
    /// Height in px.
    pub height: u32,
    /// The part of the frame that changed.
    pub damage: Damage,
    /// Exports, missed external frames, stats (shader_failures is always empty on the CPU).
    pub report: FrameReport,
}

/// One image drawn without a window, with its report.
pub struct Rendered {
    /// The image, premultiplied RGBA8.
    pub image: CpuFrame,
    /// Exports, missed external frames and stats.
    pub report: FrameReport,
}

/// The last export of a sink: what the consumer has.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ExportRecord {
    /// The request answered.
    generation: u64,
    /// How the frame was delivered.
    mode: ExportMode,
    /// Size in px.
    size: (u32, u32),
    /// Device px per logical px.
    scale: f64,
    /// Hash of the frame's pixels.
    content: u64,
}

/// The pixel stores kept between frames, next to the pure cache index.
#[derive(Default)]
struct Stores {
    /// The chunk pixels by key.
    chunks: Chunks,
    /// The kept layer surfaces by layer.
    surfaces: HashMap<LayerKey, Pixmap>,
    /// The last export of each sink.
    exports: HashMap<DrawableId, ExportRecord>,
}

impl Stores {
    /// Drop the pixels whose records the cache no longer holds, and the exports of sinks that are gone.
    fn prune(&mut self, cache: &CacheIndex, composition: &Composition) {
        self.chunks.retain(|key, _| cache.chunks.contains_key(key));
        self.surfaces
            .retain(|key, _| cache.layers.contains_key(key));
        self.exports
            .retain(|sink, _| composition.exports.iter().any(|e| e.sink == *sink));
    }
}

/// What one plan execution produced.
struct Executed {
    /// The window frame.
    output: Pixmap,
    /// The damage of the frame.
    damage: Damage,
    /// The report of the frame.
    report: FrameReport,
    /// The frame number the records carry.
    frame: u64,
    /// The chunk records to store.
    chunk_records: Vec<(ChunkKey, ChunkRecord)>,
    /// The layer records to store.
    layer_records: Vec<(LayerKey, LayerRecord)>,
}

/// Counts of the rasters of a frame.
#[derive(Default)]
struct Counts {
    /// Chunks rastered.
    chunks: u32,
    /// Pixels rastered.
    pixels: u64,
}

/// The CPU compositor: a rasterizer and the frame state kept between frames.
pub struct CpuCompositor {
    /// The chunk rasterizer.
    raster: CpuRaster,
    /// The previous frame's records.
    cache: CacheIndex,
    /// The window frame, kept across frames.
    output: Pixmap,
    /// The pixels the cache refers to.
    stores: Stores,
}

impl Default for CpuCompositor {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuCompositor {
    /// A compositor with no frame yet.
    pub fn new() -> CpuCompositor {
        CpuCompositor {
            raster: CpuRaster::new(),
            cache: CacheIndex::default(),
            output: Pixmap::default(),
            stores: Stores::default(),
        }
    }

    /// Release rebuildable raster metadata (font validation), keeping the frame state.
    pub fn release_caches(&mut self) {
        self.raster.release_caches();
    }

    /// Whether the next frame must redraw everything (after a resize or a lost buffer).
    pub fn invalidate(&mut self) {
        self.cache.clear();
        self.stores = Stores::default();
        self.output = Pixmap::default();
    }

    /// One window frame of physical `size` at `scale`.
    pub fn frame(
        &mut self,
        composition: &Composition,
        size: (u32, u32),
        scale: f64,
    ) -> Result<CpuOutput<'_>, ComposeError> {
        if self.output.size() != size {
            self.invalidate();
            self.output = Pixmap::new(size.0, size.1);
        }
        let output = self.output.take();
        let done = match run(
            &mut self.raster,
            &self.cache,
            &mut self.stores,
            output,
            composition,
            size,
            scale,
        ) {
            Ok(done) => done,
            Err(error) => {
                self.invalidate();
                return Err(error);
            }
        };
        self.cache
            .store(done.frame, done.chunk_records, done.layer_records);
        self.stores.prune(&self.cache, composition);
        self.output = done.output;
        Ok(CpuOutput {
            rgba: &self.output.data,
            width: size.0,
            height: size.1,
            damage: done.damage,
            report: done.report,
        })
    }

    /// A one-off image of `composition` (snapshot, Offscreen), with a fresh cache.
    pub fn render(
        &mut self,
        composition: &Composition,
        size: (u32, u32),
        scale: f64,
    ) -> Result<Rendered, ComposeError> {
        let mut stores = Stores::default();
        let done = run(
            &mut self.raster,
            &CacheIndex::default(),
            &mut stores,
            Pixmap::new(size.0, size.1),
            composition,
            size,
            scale,
        )?;
        Ok(Rendered {
            image: CpuFrame {
                width: size.0,
                height: size.1,
                rgba: done.output.data,
            },
            report: done.report,
        })
    }
}

/// Plans and executes one frame; the caller stores the records of a successful frame.
fn run(
    raster: &mut CpuRaster,
    cache: &CacheIndex,
    stores: &mut Stores,
    output: Pixmap,
    composition: &Composition,
    size: (u32, u32),
    scale: f64,
) -> Result<Executed, ComposeError> {
    let plan = plan::build(composition, size, scale, cache, false)
        .map_err(|error| ComposeError::InvalidComposition(error.message))?;
    let mut counts = Counts::default();
    for task in &plan.chunks {
        raster_chunk(raster, task, &mut stores.chunks, &mut counts)?;
    }
    let mut targets = make_targets(&plan.targets, output, &mut stores.surfaces);
    let mut missed: Vec<ExternalFrame> = Vec::new();
    for op in &plan.ops {
        exec(op, &mut targets, &stores.chunks, &mut missed)?;
    }
    let exports = export_frames(composition, &plan.exports, &targets, stores);
    keep_surfaces(&plan.targets, &mut targets, &mut stores.surfaces);
    let output = std::mem::take(&mut targets[0]);
    let mut stats: FrameStats = plan.stats;
    stats.chunks_rastered = counts.chunks;
    stats.pixels_rastered = counts.pixels;
    Ok(Executed {
        output,
        damage: plan.damage,
        report: FrameReport {
            exports: exports.frames,
            missed,
            shader_failures: Vec::new(),
            pending_exports: exports.pending,
            stats,
        },
        frame: plan.frame,
        chunk_records: plan.chunk_records,
        layer_records: plan.layer_records,
    })
}

/// Rasters one chunk task into the chunk store: the whole chunk, or only its changed rectangles.
fn raster_chunk(
    raster: &mut CpuRaster,
    task: &ChunkTask<'_>,
    chunks: &mut Chunks,
    counts: &mut Counts,
) -> Result<(), ComposeError> {
    let (w, h) = task.size;
    let size = (
        u16::try_from(w).map_err(|_| RenderError::InvalidSize)?,
        u16::try_from(h).map_err(|_| RenderError::InvalidSize)?,
    );
    let scene: &RenderScene = &task.scene;
    counts.chunks += 1;
    if let Redraw::Rects(rects) = &task.redraw
        && let Some(mut px) = chunks.remove(&task.key)
        && px.size() == task.size
    {
        for rect in rects {
            raster.raster(
                scene,
                task.origin,
                task.scale,
                size,
                Some(*rect),
                &mut px.data,
            )?;
            counts.pixels += area(rect);
        }
        chunks.insert(task.key, px);
        return Ok(());
    }
    let mut px = Pixmap::new(w, h);
    raster.raster(scene, task.origin, task.scale, size, None, &mut px.data)?;
    counts.pixels += u64::from(w) * u64::from(h);
    chunks.insert(task.key, px);
    Ok(())
}

/// The pixels of a rectangle.
fn area(rect: &PixelRect) -> u64 {
    u64::from(rect.width) * u64::from(rect.height)
}

/// The targets of the frame: the output, kept surfaces that are reused, and new transparent pixmaps.
fn make_targets(
    specs: &[plan::TargetSpec],
    output: Pixmap,
    surfaces: &mut HashMap<LayerKey, Pixmap>,
) -> Vec<Pixmap> {
    let mut targets = Vec::with_capacity(specs.len());
    targets.push(output);
    for spec in specs.iter().skip(1) {
        let px = match spec.kind {
            TargetKind::Kept {
                key, reuse: true, ..
            } => surfaces
                .remove(&key)
                .filter(|px| px.size() == spec.size)
                .unwrap_or_else(|| Pixmap::new(spec.size.0, spec.size.1)),
            _ => Pixmap::new(spec.size.0, spec.size.1),
        };
        targets.push(px);
    }
    targets
}

/// Stores the kept surfaces of the frame for the next frame.
fn keep_surfaces(
    specs: &[plan::TargetSpec],
    targets: &mut [Pixmap],
    surfaces: &mut HashMap<LayerKey, Pixmap>,
) {
    for (i, spec) in specs.iter().enumerate().skip(1) {
        if let TargetKind::Kept { key, .. } = spec.kind {
            surfaces.insert(key, std::mem::take(&mut targets[i]));
        }
    }
}

/// Executes one operation of the plan.
fn exec(
    op: &Op<'_>,
    targets: &mut [Pixmap],
    chunks: &Chunks,
    missed: &mut Vec<ExternalFrame>,
) -> Result<(), ComposeError> {
    match op {
        Op::Clear { target, rect } => targets[*target as usize].clear(*rect),
        Op::Draw(draw_op) => draw::draw(draw_op, targets, chunks, missed)?,
        Op::Filter {
            input,
            output,
            filter,
            px_per_unit,
            origin,
            ..
        } => {
            let source = targets[*input as usize].clone();
            targets[*output as usize] = filters::apply(filter, &source, *px_per_unit, *origin)?;
        }
        Op::Mask { target, masks } => {
            let mut layers = Vec::with_capacity(masks.len());
            for (key, mode, composite) in masks {
                layers.push((chunk(chunks, key)?, *mode, *composite));
            }
            mask::masks(&mut targets[*target as usize], &layers);
        }
        Op::Clip { target, coverage } => {
            mask::clip(&mut targets[*target as usize], chunk(chunks, coverage)?);
        }
        // The CPU draws the shader's fallback chunk in place of the shader (design §8.3, §11.3.6).
        Op::Shader(shader) => {
            let fallback = DrawOp {
                target: shader.target,
                source: Source::Chunk(shader.fallback),
                polygon: shader.polygon.clone(),
                split_edges: Vec::new(),
                map: shader.fallback_map,
                opacity: shader.opacity,
                blend: BlendMode::Normal,
                composite: Composite::SourceOver,
                sampling: Sampling::Smooth,
                coverage: Some(shader.coverage),
                scissor: Scissor::Off,
            };
            draw::draw(&fallback, targets, chunks, missed)?;
        }
    }
    Ok(())
}

/// The chunk `key` of this frame or a kept one.
fn chunk<'a>(chunks: &'a Chunks, key: &ChunkKey) -> Result<&'a Pixmap, ComposeError> {
    chunks.get(key).ok_or_else(|| draw::missing_chunk(*key))
}

/// The exports of the frame: pixel frames when their content changed, and the GPU exports the CPU cannot send.
struct ExportOutcome {
    /// Frames sent.
    frames: Vec<ExportedFrame>,
    /// Sinks whose GPU export waits for the next frame.
    pending: Vec<DrawableId>,
}

/// Decides and builds the export frames of the frame (design §8.12, the CPU part).
fn export_frames(
    composition: &Composition,
    exports: &[plan::PlannedExport],
    targets: &[Pixmap],
    stores: &mut Stores,
) -> ExportOutcome {
    let mut outcome = ExportOutcome {
        frames: Vec::new(),
        pending: Vec::new(),
    };
    for (export, planned) in composition.exports.iter().zip(exports) {
        let (sink, target) = (planned.sink, planned.target);
        if export.mode == ExportMode::Gpu {
            outcome.pending.push(sink);
            continue;
        }
        let px = &targets[target as usize];
        let record = ExportRecord {
            generation: export.generation,
            mode: export.mode,
            size: export.size,
            scale: export.scale,
            content: content_hash(px),
        };
        if stores.exports.get(&sink) == Some(&record) {
            continue;
        }
        stores.exports.insert(sink, record);
        outcome.frames.push(ExportedFrame {
            sink,
            generation: export.generation,
            frame: ExportFrame::Pixels(unpremultiplied(px)),
        });
    }
    outcome
}

/// A hash of a pixmap's bytes (deterministic within a process).
fn content_hash(px: &Pixmap) -> u64 {
    let mut hasher = DefaultHasher::new();
    px.data.hash(&mut hasher);
    hasher.finish()
}

/// The straight-alpha RGBA8 pixels of a premultiplied pixmap.
fn unpremultiplied(px: &Pixmap) -> Pixels {
    let (w, h) = px.size();
    let mut out = Vec::with_capacity(px.data.len());
    for p in px.data.as_chunks::<4>().0 {
        let a = f32::from(p[3]) / 255.0;
        if a <= 0.0 {
            out.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }
        for c in &p[..3] {
            out.push((f32::from(*c) / a).round().min(255.0) as u8);
        }
        out.push(p[3]);
    }
    Pixels::new(w, h, out)
}
