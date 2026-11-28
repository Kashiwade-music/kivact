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

use super::cull::keep_mask;
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
    /// The pixels the ops visited or copied (tests only).
    #[cfg(test)]
    composited: u64,
}

/// Counts of the rasters of a frame.
#[derive(Default)]
struct Counts {
    /// Chunks rastered.
    chunks: u32,
    /// Pixels rastered.
    pixels: u64,
    /// Pixels the ops visited or copied (tests only).
    #[cfg(test)]
    composited: u64,
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
    /// The pixels the last frame's draws visited and copies wrote (tests only).
    #[cfg(test)]
    composited: u64,
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
            #[cfg(test)]
            composited: 0,
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
        #[cfg(test)]
        {
            self.composited = done.composited;
        }
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
        #[cfg(test)]
        {
            counts.composited += written_pixels(op, &targets);
        }
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
        #[cfg(test)]
        composited: counts.composited,
    })
}

/// The pixels an op visits (a draw's spans in its target) or copies (a copy's rectangles), clipped to the target (tests only).
#[cfg(test)]
fn written_pixels(op: &Op<'_>, targets: &[Pixmap]) -> u64 {
    match op {
        Op::Draw(draw_op) => {
            let Some((x0, x1, y0, y1)) = plan::pixel_rect(&draw_op.polygon.points) else {
                return 0;
            };
            let size = targets[draw_op.target as usize].size();
            let bounds = draw::clip_span((x0, x1, y0, y1), size);
            draw::spans(bounds, &draw_op.scissor)
                .into_iter()
                .map(span_area)
                .sum()
        }
        Op::Copy { target, rects, .. } => {
            let size = targets[*target as usize].size();
            rects
                .iter()
                .map(|r| {
                    let (x, y) = (i64::from(r.x), i64::from(r.y));
                    let span = (x, x + i64::from(r.width), y, y + i64::from(r.height));
                    span_area(draw::clip_span(span, size))
                })
                .sum()
        }
        _ => 0,
    }
}

/// The area of a span, zero when it is empty.
#[cfg(test)]
fn span_area((x0, x1, y0, y1): draw::Span) -> u64 {
    u64::from(x1.saturating_sub(x0)) * u64::from(y1.saturating_sub(y0))
}

/// Rasters one chunk task into the chunk store: a fresh chunk in its drawn parts, a redrawn one in
/// its changed rectangles, or else the whole chunk.
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
    if let Redraw::Fresh(parts) = &task.redraw {
        let mut px = Pixmap::new(w, h);
        raster_rects(raster, task, parts, size, &mut px, counts)?;
        chunks.insert(task.key, px);
        return Ok(());
    }
    if let Redraw::Rects(rects) = &task.redraw
        && let Some(mut px) = chunks.remove(&task.key)
        && px.size() == task.size
    {
        raster_rects(raster, task, rects, size, &mut px, counts)?;
        chunks.insert(task.key, px);
        return Ok(());
    }
    let mut px = Pixmap::new(w, h);
    let whole = PixelRect {
        x: task.offset.0,
        y: task.offset.1,
        width: w,
        height: h,
    };
    let keep = keep_mask(scene, &task.index, whole);
    raster.raster(
        scene,
        task.origin,
        task.scale,
        size,
        None,
        &mut px.data,
        Some(keep.as_slice()),
    )?;
    counts.pixels += u64::from(w) * u64::from(h);
    chunks.insert(task.key, px);
    Ok(())
}

/// Rasters the chunk px `rects` of `task` into `px`, each with the mask of its commands.
fn raster_rects(
    raster: &mut CpuFrameRaster<'_>,
    task: &ChunkTask<'_>,
    rects: &[PixelRect],
    size: (u16, u16),
    px: &mut Pixmap,
    counts: &mut Counts,
) -> Result<(), RenderError> {
    let scene: &RenderScene = &task.scene;
    let masks: Vec<Vec<bool>> = rects
        .iter()
        .map(|r| {
            let region = PixelRect {
                x: r.x + task.offset.0,
                y: r.y + task.offset.1,
                ..*r
            };
            keep_mask(scene, &task.index, region)
        })
        .collect();
    let parts: Vec<(PixelRect, &[bool])> = rects
        .iter()
        .zip(&masks)
        .map(|(r, m)| (*r, m.as_slice()))
        .collect();
    raster.raster_parts(scene, task.origin, task.scale, size, &parts, &mut px.data)?;
    counts.pixels += rects.iter().map(area).sum::<u64>();
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
        Op::Copy {
            target,
            source,
            offset,
            rects,
        } => {
            targets[*target as usize].copy_rects_from(chunk(chunks, source)?, *offset, rects);
        }
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
    use arvane_protocol::{
        CompositeLayer, ExternalItem, LayerEffects, LayerItem, PieceKey, Plane, RasterGrid,
        RasterPaint, RenderBackdrop, RenderCmd,
    };
    use kivact_types::Rgba;
    use kivact_types::geometry::{Affine, Rect};
    use kivact_types::geometry3d::Matrix4;
    use kivact_types::gpu::ExternalImageId;
    use kivact_types::ids::NodeId;
    use kivact_types::paint::BackfaceVisibility;
    use kivact_types::scene::Outline;

    use super::*;

    /// A composition of one root layer of `size` px whose only item is `scene`.
    fn root_only(scene: RenderScene, content_version: u64, size: (u32, u32)) -> Composition {
        let root = CompositeLayer {
            key: LayerKey::Root,
            transform: Matrix4::IDENTITY,
            bounds: Rect::new(0.0, 0.0, f64::from(size.0), f64::from(size.1)),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects::default(),
            content_version,
            items: vec![LayerItem::Raster {
                scene,
                key: PieceKey {
                    node: NodeId(0),
                    n: 0,
                },
            }],
        };
        Composition {
            layers: vec![root],
            exports: Vec::new(),
        }
    }

    /// 3 000 squares of 8 × 8 px, one per cell of a 60 × 50 grid of 32 × 20 px
    /// cells; the square `recolored` (an index) is drawn in another colour.
    fn grid_scene(recolored: Option<usize>) -> RenderScene {
        let commands = (0..3000)
            .map(|i| {
                let (col, row) = (i % 60, i / 60);
                let red = if recolored == Some(i) { 0.9 } else { 0.2 };
                RenderCmd::FillShape {
                    shape: Outline::Rect(Rect::new(col as f64 * 32.0, row as f64 * 20.0, 8.0, 8.0)),
                    paint: RasterPaint::Solid(Rgba {
                        r: red,
                        g: 0.5,
                        b: 0.3,
                        a: 1.0,
                    }),
                }
            })
            .collect();
        RenderScene { commands }
    }

    /// A frame that changes one small square of a large raster composites only that square.
    #[test]
    fn a_small_change_composites_only_its_damage() {
        let size = (2048, 1024);
        let mut compositor = CpuCompositor::new();
        compositor
            .frame(&root_only(grid_scene(None), 1, size), size, 1.0)
            .expect("frame 1 draws");
        let damage_pixels = compositor
            .frame(&root_only(grid_scene(Some(1234)), 2, size), size, 1.0)
            .expect("frame 2 draws")
            .report
            .stats
            .damage_pixels;
        assert_eq!(damage_pixels, 64, "one 8 × 8 square changed");
        assert!(
            compositor.composited <= 4 * damage_pixels,
            "frame 2 composited {} px for {damage_pixels} px of damage",
            compositor.composited
        );
    }

    /// A full frame of a root-only composition is one copy of the root's chunk, with no clear.
    #[test]
    fn a_full_frame_is_a_copy() {
        // The squares reach 1 896 px across: the root's cell is the whole 1 920 px window.
        let size = (1920, 1024);
        let composition = root_only(grid_scene(None), 1, size);
        let cache = CacheIndex::default();
        let plan = plan::build(&composition, size, 1.0, &cache, false).expect("the root plans");
        assert!(
            plan.ops.iter().any(|op| matches!(op, Op::Copy { .. })),
            "a full frame copies its chunk onto the output"
        );
        assert!(
            !plan.ops.iter().any(|op| matches!(op, Op::Clear { .. })),
            "the copy replaces the output's clear"
        );
        let mut compositor = CpuCompositor::new();
        compositor
            .frame(&composition, size, 1.0)
            .expect("the root draws");
        let chunks: Vec<&Pixmap> = compositor.stores.chunks.values().collect();
        assert_eq!(chunks.len(), 1, "the root raster is one chunk");
        assert!(
            compositor.output.data == chunks[0].data,
            "the output bytes equal the chunk bytes"
        );
    }

    /// An 800 × 600 root of a white raster and a default-effects layer holding the L shape of the
    /// e2e test: a panel along the right edge and a bar along the bottom that overlaps it by 5 px.
    fn l_layers() -> Composition {
        let fill = |r: Rect| RenderCmd::FillShape {
            shape: Outline::Rect(r),
            paint: RasterPaint::Solid(Rgba {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }),
        };
        let key = |n: u32| PieceKey { node: NodeId(0), n };
        let layer = |key_of: LayerKey, items: Vec<LayerItem>| CompositeLayer {
            key: key_of,
            transform: Matrix4::IDENTITY,
            bounds: Rect::new(0.0, 0.0, 800.0, 600.0),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects::default(),
            content_version: 1,
            items,
        };
        let white = RenderScene {
            commands: vec![RenderCmd::FillShape {
                shape: Outline::Rect(Rect::new(0.0, 0.0, 800.0, 600.0)),
                paint: RasterPaint::Solid(Rgba {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
            }],
        };
        let l = RenderScene {
            commands: vec![
                fill(Rect::new(600.0, 0.0, 200.0, 580.0)),
                fill(Rect::new(0.0, 575.0, 800.0, 25.0)),
            ],
        };
        let root = layer(
            LayerKey::Root,
            vec![
                LayerItem::Raster {
                    scene: white,
                    key: key(0),
                },
                LayerItem::Layer {
                    index: 1,
                    clips: Vec::new(),
                    key: key(1),
                },
            ],
        );
        let child = layer(
            LayerKey::Node(NodeId(2)),
            vec![LayerItem::Raster {
                scene: l,
                key: key(0),
            }],
        );
        Composition {
            layers: vec![root, child],
            exports: Vec::new(),
        }
    }

    /// A chunk is composited only over the clusters it draws, not over its whole cell.
    #[test]
    fn a_chunk_composites_only_what_it_draws() {
        let size = (800, 600);
        let mut compositor = CpuCompositor::new();
        compositor
            .frame(&l_layers(), size, 1.0)
            .expect("the frame draws");
        assert!(
            compositor.composited <= 480_000 + 140_000,
            "the L chunk was composited over its cell: {} px",
            compositor.composited
        );
    }

    /// A plan that reads the output before its first writer keeps the output's clear and has no copy.
    #[test]
    fn an_early_read_of_the_output_keeps_the_clears() {
        let size = (64, 64);
        let root = CompositeLayer {
            key: LayerKey::Root,
            transform: Matrix4::IDENTITY,
            bounds: Rect::new(0.0, 0.0, 64.0, 64.0),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects::default(),
            content_version: 1,
            items: vec![
                LayerItem::Layer {
                    index: 1,
                    clips: Vec::new(),
                    key: PieceKey {
                        node: NodeId(0),
                        n: 0,
                    },
                },
                LayerItem::Raster {
                    scene: RenderScene::default(),
                    key: PieceKey {
                        node: NodeId(0),
                        n: 1,
                    },
                },
            ],
        };
        // The child's backdrop reads what lies behind it: the output, before the root's raster draws it.
        let child = CompositeLayer {
            key: LayerKey::Node(NodeId(1)),
            transform: Matrix4::IDENTITY,
            bounds: Rect::new(8.0, 8.0, 32.0, 32.0),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects {
                backdrop: Some(RenderBackdrop {
                    shape: Outline::Rect(Rect::new(8.0, 8.0, 32.0, 32.0)),
                    filters: Vec::new(),
                }),
                ..LayerEffects::default()
            },
            content_version: 1,
            items: vec![LayerItem::Raster {
                scene: RenderScene::default(),
                key: PieceKey {
                    node: NodeId(0),
                    n: 0,
                },
            }],
        };
        let composition = Composition {
            layers: vec![root, child],
            exports: Vec::new(),
        };
        let cache = CacheIndex::default();
        let plan = plan::build(&composition, size, 1.0, &cache, false).expect("the layers plan");
        assert!(
            plan.ops
                .iter()
                .any(|op| matches!(op, Op::Draw(draw) if matches!(draw.source, Source::Target(0)))),
            "the backdrop reads the output"
        );
        assert!(
            plan.ops
                .iter()
                .any(|op| matches!(op, Op::Clear { target: 0, .. })),
            "the output keeps its clear"
        );
        assert!(
            !plan.ops.iter().any(|op| matches!(op, Op::Copy { .. })),
            "a plan that reads the output early has no copy"
        );
    }

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
                key: PieceKey {
                    node: NodeId(0),
                    n: 0,
                },
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
            items: vec![LayerItem::Raster {
                scene: RenderScene::default(),
                key: PieceKey {
                    node: NodeId(0),
                    n: 0,
                },
            }],
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

    /// A 64 × 64 root of a white raster and, while `with`, an external item of 16 × 16 red CPU pixels at (8, 8).
    fn with_external(with: bool) -> Composition {
        let white = RenderScene {
            commands: vec![RenderCmd::FillShape {
                shape: Outline::Rect(Rect::new(0.0, 0.0, 64.0, 64.0)),
                paint: RasterPaint::Solid(Rgba {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
            }],
        };
        let mut items = vec![LayerItem::Raster {
            scene: white,
            key: PieceKey {
                node: NodeId(0),
                n: 0,
            },
        }];
        if with {
            items.push(LayerItem::External(ExternalItem {
                frame: ExternalFrame {
                    image: ExternalImageId(1),
                    slot: 0,
                    value: 1,
                },
                rect: Rect::new(8.0, 8.0, 16.0, 16.0),
                source: None,
                transform: Affine::IDENTITY,
                clips: Vec::new(),
                opacity: 1.0,
                pixelated: true,
                cpu: Some(Pixels::new(16, 16, [255u8, 0, 0, 255].repeat(256))),
                key: PieceKey {
                    node: NodeId(0),
                    n: 1,
                },
            }));
        }
        let root = CompositeLayer {
            key: LayerKey::Root,
            transform: Matrix4::IDENTITY,
            bounds: Rect::new(0.0, 0.0, 64.0, 64.0),
            plane: Plane::Flat,
            backface: BackfaceVisibility::Visible,
            grid: RasterGrid::Target,
            effects: LayerEffects::default(),
            content_version: 1,
            items,
        };
        Composition {
            layers: vec![root],
            exports: Vec::new(),
        }
    }

    /// An external item that goes away leaves nothing on the output.
    #[test]
    fn a_removed_external_leaves_nothing() {
        let size = (64, 64);
        let mut compositor = CpuCompositor::new();
        compositor
            .frame(&with_external(true), size, 1.0)
            .expect("frame 1 draws");
        compositor
            .frame(&with_external(false), size, 1.0)
            .expect("frame 2 draws");
        let mut fresh = CpuCompositor::new();
        fresh
            .frame(&with_external(false), size, 1.0)
            .expect("the fresh frame draws");
        let differ = compositor
            .output
            .data
            .chunks(4)
            .zip(fresh.output.data.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            differ, 0,
            "after the removed external: {differ} pixels differ"
        );
    }
}
