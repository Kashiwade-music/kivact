//! Role: CpuCompositor: runs a planned frame on the calling thread.

use std::collections::HashMap;

use arvane_protocol::{
    Composition, CpuFrame, ExportMode, FrameReport, LayerFilter, LayerKey, PixelRect, RenderScene,
};
use arvane_render::{CpuFrameRaster, CpuRaster, RenderError};
use kivact_types::Composite;
use kivact_types::export::{ExportFrame, ExportedFrame, FrameStats};
use kivact_types::gpu::ExternalFrame;
use kivact_types::ids::DrawableId;
use kivact_types::pixel::{BlendMode, Pixels};

use super::draw::{self, Chunks};
use super::filters;
use super::mask;
use super::surface::Pixmap;
use crate::cache::{CacheIndex, ChunkKey, ChunkRecord, LayerRecord};
use crate::plan::{self, ChunkTask, Damage, DrawOp, Op, Redraw, Sampling, Source, TargetKind};

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
    /// The planned signature of the layer the frame was made from.
    signature: u64,
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
        self.chunks.retain(|key, _| cache.keeps_chunk(key));
        self.surfaces.retain(|key, _| cache.keeps_layer(key));
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
    /// The chunks whose old records to drop.
    chunk_forgets: Vec<ChunkKey>,
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
    /// Temporary targets kept for reuse by size, at most POOL_BYTES in all.
    pool: Vec<Pixmap>,
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
            pool: Vec::new(),
        }
    }

    /// Release rebuildable raster metadata (font validation) and the pooled
    /// targets, keeping the frame state.
    pub fn release_caches(&mut self) {
        self.raster.release_caches();
        self.pool = Vec::new();
    }

    /// Whether the next frame must redraw everything (after a resize, a lost
    /// buffer, or when the window has been idle). The pooled targets are freed
    /// with the rest (architecture.md MEM-8).
    pub fn invalidate(&mut self) {
        self.cache.clear();
        self.stores = Stores::default();
        self.output = Pixmap::default();
        self.pool = Vec::new();
        self.raster.forget_scratch();
    }

    /// One window frame of physical `size` at `scale`.
    pub fn frame(
        &mut self,
        composition: &Composition,
        size: (u32, u32),
        scale: f64,
    ) -> Result<CpuOutput<'_>, RenderError> {
        if self.output.size() != size {
            self.invalidate();
            self.output = Pixmap::new(size.0, size.1);
        }
        let output = self.output.take();
        let done = match run(
            &mut self.raster,
            &mut self.pool,
            &self.cache,
            &mut self.stores,
            output,
            composition,
            scale,
        ) {
            Ok(done) => done,
            Err(error) => {
                self.invalidate();
                return Err(error);
            }
        };
        self.cache.store(
            done.frame,
            done.chunk_records,
            done.layer_records,
            &done.chunk_forgets,
        );
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
    ) -> Result<Rendered, RenderError> {
        let mut stores = Stores::default();
        let mut pool = Vec::new();
        let done = run(
            &mut self.raster,
            &mut pool,
            &CacheIndex::default(),
            &mut stores,
            Pixmap::new(size.0, size.1),
            composition,
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

/// Plans and executes one frame of the output's size; the caller stores the records of a successful frame.
fn run(
    raster: &mut CpuRaster,
    pool: &mut Vec<Pixmap>,
    cache: &CacheIndex,
    stores: &mut Stores,
    output: Pixmap,
    composition: &Composition,
    scale: f64,
) -> Result<Executed, RenderError> {
    let plan = plan::build(composition, output.size(), scale, cache, false)
        .map_err(|error| RenderError::InvalidComposition(error.message))?;
    let mut counts = Counts::default();
    raster.frame(|frame| -> Result<(), RenderError> {
        for task in &plan.chunks {
            raster_chunk(frame, task, &mut stores.chunks, &mut counts)?;
        }
        Ok(())
    })?;
    let mut targets = make_targets(&plan.targets, output, &mut stores.surfaces, pool);
    let mut missed: Vec<ExternalFrame> = Vec::new();
    let mut shader_fallback = false;
    for op in &plan.ops {
        shader_fallback |= matches!(
            op,
            Op::Shader(_)
                | Op::Filter {
                    filter: LayerFilter::Shader(_),
                    ..
                }
        );
        exec(op, &mut targets, &stores.chunks, &mut missed)?;
    }
    let exports = export_frames(composition, &plan.exports, &targets, stores);
    keep_surfaces(&plan.targets, &mut targets, &mut stores.surfaces);
    let output = std::mem::take(&mut targets[0]);
    give_back(&plan.targets, &mut targets, pool);
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
            shader_fallback,
            pending_exports: exports.pending,
            stats,
        },
        frame: plan.frame,
        chunk_records: plan.chunk_records,
        chunk_forgets: plan.chunk_forgets,
        layer_records: plan.layer_records,
    })
}

/// Rasters one chunk task into the chunk store: the whole chunk, or only its changed rectangles.
fn raster_chunk(
    raster: &mut CpuFrameRaster<'_>,
    task: &ChunkTask<'_>,
    chunks: &mut Chunks,
    counts: &mut Counts,
) -> Result<(), RenderError> {
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
        raster.raster_rects(scene, task.origin, task.scale, size, rects, &mut px.data)?;
        counts.pixels += rects.iter().map(area).sum::<u64>();
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

/// The most bytes of temporary targets kept between frames.
const POOL_BYTES: usize = 64 * 1024 * 1024;

/// A cleared target of `size`, reused from the pool when one has that size.
fn take_target(pool: &mut Vec<Pixmap>, size: (u32, u32)) -> Pixmap {
    match pool.iter().position(|px| px.size() == size) {
        Some(at) => {
            let mut px = pool.swap_remove(at);
            px.clear(None);
            px
        }
        None => Pixmap::new(size.0, size.1),
    }
}

/// Keep `px` for reuse while the pool stays within `POOL_BYTES`; a target
/// that does not fit is freed (a large window's targets are not kept).
fn give_target(pool: &mut Vec<Pixmap>, px: Pixmap) {
    let kept: usize = pool.iter().map(|p| p.data.len()).sum();
    if kept + px.data.len() <= POOL_BYTES {
        pool.push(px);
    }
}

/// The targets of the frame: the output, kept surfaces that are reused, and pooled or new transparent pixmaps.
fn make_targets(
    specs: &[plan::TargetSpec],
    output: Pixmap,
    surfaces: &mut HashMap<LayerKey, Pixmap>,
    pool: &mut Vec<Pixmap>,
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
            _ => take_target(pool, spec.size),
        };
        targets.push(px);
    }
    targets
}

/// Returns the scratch targets of the frame to the pool (the output and the
/// kept surfaces are not scratch: their pixels were moved out).
fn give_back(specs: &[plan::TargetSpec], targets: &mut [Pixmap], pool: &mut Vec<Pixmap>) {
    for (spec, px) in specs.iter().zip(targets.iter_mut()).skip(1) {
        if matches!(spec.kind, TargetKind::Temp) {
            give_target(pool, std::mem::take(px));
        }
    }
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
) -> Result<(), RenderError> {
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
            if input == output {
                let source = targets[*input as usize].clone();
                let out = &mut targets[*output as usize];
                filters::apply(filter, &source, out, *px_per_unit, *origin)?;
            } else {
                let (source, out) = source_and_out(targets, *input as usize, *output as usize);
                filters::apply(filter, source, out, *px_per_unit, *origin)?;
            }
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
        // The CPU draws the shader's fallback chunk in place of the shader.
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
                scissor: shader.scissor.clone(),
            };
            draw::draw(&fallback, targets, chunks, missed)?;
        }
    }
    Ok(())
}

/// The target `input` to read and the target `output` to write (`input != output`).
fn source_and_out(targets: &mut [Pixmap], input: usize, output: usize) -> (&Pixmap, &mut Pixmap) {
    if input < output {
        let (low, high) = targets.split_at_mut(output);
        (&low[input], &mut high[0])
    } else {
        let (low, high) = targets.split_at_mut(input);
        (&high[0], &mut low[output])
    }
}

/// The chunk `key` of this frame or a kept one.
fn chunk<'a>(chunks: &'a Chunks, key: &ChunkKey) -> Result<&'a Pixmap, RenderError> {
    chunks.get(key).ok_or_else(|| draw::missing_chunk(*key))
}

/// The exports of the frame: pixel frames when their content changed, and the GPU exports the CPU cannot send.
struct ExportOutcome {
    /// Frames sent.
    frames: Vec<ExportedFrame>,
    /// Sinks whose GPU export waits for the next frame.
    pending: Vec<DrawableId>,
}

/// Decides and builds the export frames of the frame (the CPU part).
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
            signature: planned.signature,
        };
        if stores.exports.get(&sink) == Some(&record) {
            continue;
        }
        stores.exports.insert(sink, record);
        outcome.frames.push(ExportedFrame {
            sink,
            generation: export.generation,
            frame: ExportFrame::Pixels(Pixels::new(px.size().0, px.size().1, px.data.clone())),
        });
    }
    outcome
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the CPU compositor's frame composition.
    use arvane_protocol::{CompositeLayer, LayerEffects, LayerItem, Plane, RasterGrid};
    use kivact_types::geometry::Rect;
    use kivact_types::geometry3d::Matrix4;
    use kivact_types::ids::NodeId;
    use kivact_types::paint::BackfaceVisibility;

    use super::*;

    /// A 64 × 64 px window: a root that draws one blurred child layer, which
    /// is drawn through a temporary target (its filter's input).
    fn blurred_child() -> Composition {
        let root = CompositeLayer {
            key: LayerKey::Root,
            transform: Matrix4::IDENTITY,
            bounds: Rect::new(0.0, 0.0, 64.0, 64.0),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects::default(),
            content_version: 1,
            items: vec![LayerItem::Layer {
                index: 1,
                clips: Vec::new(),
            }],
        };
        let child = CompositeLayer {
            key: LayerKey::Node(NodeId(1)),
            transform: Matrix4::IDENTITY,
            bounds: Rect::new(8.0, 8.0, 32.0, 32.0),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects {
                filters: vec![LayerFilter::Blur { std_dev: 2.0 }],
                ..LayerEffects::default()
            },
            content_version: 1,
            items: vec![LayerItem::Raster(RenderScene::default())],
        };
        Composition {
            layers: vec![root, child],
            exports: Vec::new(),
        }
    }

    #[test]
    fn the_target_pool_keeps_at_most_64_mib() {
        let size = (4096, 2048);
        let mut pool = Vec::new();
        for _ in 0..3 {
            give_target(&mut pool, Pixmap::new(size.0, size.1));
        }
        assert_eq!(pool.len(), 2, "two targets of 32 MiB fit in 64 MiB");
        let pooled: Vec<*const u8> = pool.iter().map(|px| px.data.as_ptr()).collect();
        let first = take_target(&mut pool, size);
        let second = take_target(&mut pool, size);
        assert!(pool.is_empty(), "both pooled targets were taken");
        assert!(pooled.contains(&first.data.as_ptr()));
        assert!(pooled.contains(&second.data.as_ptr()));
        let third = take_target(&mut pool, size);
        assert_eq!(third.size(), size);
        assert!(pool.is_empty(), "a third target is made, not pooled");
        assert!(!pooled.contains(&third.data.as_ptr()));
    }

    #[test]
    fn forget_frame_state_releases_every_pooled_target() {
        let composition = blurred_child();
        let mut compositor = CpuCompositor::new();
        compositor
            .frame(&composition, (64, 64), 1.0)
            .expect("the blurred child draws");
        assert!(
            !compositor.pool.is_empty(),
            "the child's temporary target is pooled after the frame"
        );
        compositor.invalidate();
        assert!(compositor.pool.is_empty());
    }
}
