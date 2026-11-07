//! Role: Plans one composition frame into draw operations: surfaces, chunk rasters, 3D order, cache reuse and damage.
//!
//! The plan knows no backend. A CPU or GPU executor draws its [`Op`]s in
//! order into the targets it allocates from [`Plan::targets`], and rasters
//! the [`ChunkTask`]s first. Records are returned, not stored: the executor
//! stores them after a successful frame.

mod bounds;
mod context3d;
mod damage;
mod hash;
mod items;
mod layers;
mod polygon;
mod validate;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect, Size};
use kivact_types::geometry3d::{Polygon, Projective};
use kivact_types::ids::DrawableId;

use kivact_types::effect::ShaderUse;
use kivact_types::gpu::ExternalFrame;
use kivact_types::paint::{MaskComposite, MaskMode};
use kivact_types::pixel::{BlendMode, Pixels};
use kivact_types::render::{
    Composite, Composition, FrameStats, LayerFilter, LayerKey, PixelRect, Plane, RenderScene,
};

use crate::cache::{CacheIndex, ChunkKey, ChunkRecord, LayerRecord};
use items::{Dest, ItemCtx};

/// Index of a target: 0 is the output, the rest are scratch or kept surfaces of this frame.
pub(crate) type TargetId = u32;

/// The output target (the window's pixmap or the swapchain).
pub(crate) const OUTPUT: TargetId = 0;

/// Targets above which the largest layers are drawn into their parent instead of a surface (design §8.14).
pub(crate) const INLINE_TARGET_LIMIT: usize = 1024;

/// A plan that cannot be made; the executor reports it as an invalid composition.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlanError {
    /// What is wrong with the composition.
    pub(crate) message: String,
}

impl PlanError {
    /// An error with `message`.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        PlanError {
            message: message.into(),
        }
    }
}

/// The damaged part of the output window.
#[derive(Clone, Debug, PartialEq)]
pub enum Damage {
    /// Everything is redrawn.
    Full,
    /// Only these rectangles (device px) are redrawn.
    Rects(Vec<PixelRect>),
}

/// A pixel grid of a surface: the layer point of its px (0, 0), the px per layer unit and its size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Grid {
    /// The layer point at px (0, 0).
    pub(crate) origin: Point,
    /// Px per layer unit.
    pub(crate) scale: f64,
    /// Size in px.
    pub(crate) size: (u32, u32),
}

impl Grid {
    /// The map from layer space to this grid's px.
    pub(crate) fn to_px(self) -> Matrix4 {
        Matrix4::translate(-self.origin.x, -self.origin.y, 0.0)
            .then(Matrix4::scale(self.scale, self.scale, 1.0))
    }

    /// The layer-space rectangle the grid covers.
    pub(crate) fn rect(self) -> Rect {
        Rect::new(
            self.origin.x,
            self.origin.y,
            f64::from(self.size.0) / self.scale,
            f64::from(self.size.1) / self.scale,
        )
    }
}

/// What a target holds for the frame's use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TargetKind {
    /// A scratch target of this frame; it is cleared before use. The output (target 0) is also listed here.
    Temp,
    /// A layer surface kept across frames; `reuse`: its content is current
    /// and the ops that would draw it were left out.
    Kept {
        /// The layer whose surface it is.
        key: LayerKey,
        /// The surface's signature this frame.
        signature: u64,
        /// The content is current.
        reuse: bool,
    },
}

/// A target of the frame and its size in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TargetSpec {
    /// Size in px.
    pub(crate) size: (u32, u32),
    /// What it holds.
    pub(crate) kind: TargetKind,
}

/// How a chunk is rastered this frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Redraw {
    /// The whole chunk.
    All,
    /// Only these rectangles (chunk px) changed.
    Rects(Vec<PixelRect>),
}

/// A chunk to raster before the ops run.
#[derive(Clone, Debug)]
pub(crate) struct ChunkTask<'a> {
    /// The chunk.
    pub(crate) key: ChunkKey,
    /// What it draws (borrowed from the composition or built by the plan).
    pub(crate) scene: Cow<'a, RenderScene>,
    /// Layer point at chunk px (0, 0).
    pub(crate) origin: Point,
    /// Chunk px per layer unit.
    pub(crate) scale: f64,
    /// Size in px.
    pub(crate) size: (u32, u32),
    /// The part to raster.
    pub(crate) redraw: Redraw,
    /// Per command: hash and chunk px bounds, stored into the cache.
    pub(crate) index: Arc<[(u64, Option<PixelRect>)]>,
}

/// Where a draw reads its pixels from.
#[derive(Clone, Debug)]
pub(crate) enum Source<'a> {
    /// A chunk of this frame or a kept one.
    Chunk(ChunkKey),
    /// A target of this frame.
    Target(TargetId),
    /// A frame of an external render engine's image; `cpu` pixels when the CPU draws it.
    External {
        /// The frame to draw.
        frame: ExternalFrame,
        /// The CPU pixels, if any.
        cpu: Option<&'a Pixels>,
        /// The source rectangle, None for all of it.
        source: Option<kivact_types::geometry::Rect>,
    },
    /// Straight-alpha pixels (filter images and the like).
    Pixels(&'a Pixels),
}

/// How a draw samples its source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sampling {
    /// Bilinear.
    Smooth,
    /// The texel holding the sample point.
    Nearest,
}

impl Sampling {
    /// The sampling of a draw that is `nearest` (pixel aligned) or smooth.
    pub(crate) fn nearest(nearest: bool) -> Sampling {
        if nearest {
            Sampling::Nearest
        } else {
            Sampling::Smooth
        }
    }
}

/// The rectangles a draw is limited to (window damage on the output).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Scissor {
    /// No limit.
    Off,
    /// Only these rectangles (device px).
    Rects(Vec<PixelRect>),
}

/// One draw of a target's pixels from a source.
#[derive(Clone, Debug)]
pub(crate) struct DrawOp<'a> {
    /// The target drawn into.
    pub(crate) target: TargetId,
    /// What is drawn.
    pub(crate) source: Source<'a>,
    /// Destination polygon in target px (convex).
    pub(crate) polygon: Polygon,
    /// Edges of `polygon` that are BSP splits (no antialiasing, top-left rule).
    pub(crate) split_edges: Vec<usize>,
    /// Target px → source space: chunk, target and Pixels px, or for External the
    /// item rect space (the executor takes it to texels by the frame size and `source`).
    pub(crate) map: Projective,
    /// Alpha multiplier.
    pub(crate) opacity: f64,
    /// Blend with what lies beneath.
    pub(crate) blend: BlendMode,
    /// Porter–Duff operator with what lies beneath.
    pub(crate) composite: Composite,
    /// Disable smoothing.
    pub(crate) sampling: Sampling,
    /// Coverage multiplied in, in target px.
    pub(crate) coverage: Option<ChunkKey>,
    /// Pixels of the target outside these rectangles stay as they are (window damage).
    pub(crate) scissor: Scissor,
}

/// One shader draw: the GPU runs the shader, the CPU draws `fallback`.
#[derive(Clone, Debug)]
pub(crate) struct ShaderOp<'a> {
    /// The target drawn into.
    pub(crate) target: TargetId,
    /// The shader with its parameters and images.
    pub(crate) shader: &'a ShaderUse,
    /// Destination polygon in target px.
    pub(crate) polygon: Polygon,
    /// Target px → shader space.
    pub(crate) map: Projective,
    /// The shader's coordinate box size in logical px.
    pub(crate) size: Size,
    /// Shader px per logical px.
    pub(crate) px_per_unit: f64,
    /// The coverage chunk.
    pub(crate) coverage: ChunkKey,
    /// The fallback chunk the CPU draws instead.
    pub(crate) fallback: ChunkKey,
    /// Target px → fallback chunk px. The chunk already holds the item's transform,
    /// so this is the layer-to-px map of the chunk's grid composed with the target's.
    pub(crate) fallback_map: Projective,
    /// Filter shaders read this target as content.
    pub(crate) input: Option<TargetId>,
    /// Alpha multiplier.
    pub(crate) opacity: f64,
}

/// One operation of the frame, executed in order.
#[derive(Clone, Debug)]
pub(crate) enum Op<'a> {
    /// Clear `rect` of `target` (None: all of it) to transparent.
    Clear {
        /// The target.
        target: TargetId,
        /// The rectangle, None for all of it.
        rect: Option<PixelRect>,
    },
    /// Draw a source into a target.
    Draw(DrawOp<'a>),
    /// `output` = filter(`input`); both have the same size and space.
    Filter {
        /// The input target.
        input: TargetId,
        /// The output target.
        output: TargetId,
        /// The filter.
        filter: &'a LayerFilter,
        /// Px per layer unit.
        px_per_unit: f64,
        /// The layer point at px (0, 0).
        origin: Point,
    },
    /// Multiply `target` by the combined mask of `masks` (bottom layer last).
    Mask {
        /// The target.
        target: TargetId,
        /// The mask chunks with their modes.
        masks: Vec<(ChunkKey, MaskMode, MaskComposite)>,
    },
    /// Multiply `target` by the coverage of a chunk.
    Clip {
        /// The target.
        target: TargetId,
        /// The coverage chunk.
        coverage: ChunkKey,
    },
    /// A shader draw (GPU only; the CPU draws its fallback).
    Shader(ShaderOp<'a>),
}

/// The plan of one frame.
#[derive(Debug)]
pub(crate) struct Plan<'a> {
    /// Targets; index 0 is the output.
    pub(crate) targets: Vec<TargetSpec>,
    /// Chunks to make current before `ops` run.
    pub(crate) chunks: Vec<ChunkTask<'a>>,
    /// The operations in order.
    pub(crate) ops: Vec<Op<'a>>,
    /// The output's damage (computed for the GPU too; the GPU redraws all).
    pub(crate) damage: Damage,
    /// Drawables exported this frame, with the target holding each frame.
    pub(crate) exports: Vec<(DrawableId, TargetId)>,
    /// Chunk records to store after a successful frame.
    pub(crate) chunk_records: Vec<(ChunkKey, ChunkRecord)>,
    /// Layer records to store after a successful frame.
    pub(crate) layer_records: Vec<(LayerKey, LayerRecord)>,
    /// The frame number the records carry.
    pub(crate) frame: u64,
    /// The statistics this plan decides (the executor adds the raster counts).
    pub(crate) stats: FrameStats,
}

/// Plan one frame of `composition` for a window of `size` px at `scale`.
pub(crate) fn build<'a>(
    composition: &'a Composition,
    size: (u32, u32),
    scale: f64,
    cache: &'a CacheIndex,
    gpu: bool,
) -> Result<Plan<'a>, PlanError> {
    validate::check(composition, scale)?;
    let mut inline: HashSet<u32> = HashSet::new();
    loop {
        let plan = Builder::new(composition, cache, size, scale, gpu, inline.clone()).run()?;
        if plan.targets.len() <= INLINE_TARGET_LIMIT {
            return Ok(plan);
        }
        // Too many targets: draw the next largest eligible layer into its parent and plan again.
        let next = (1..composition.layers.len() as u32)
            .rev()
            .find(|i| !inline.contains(i) && layers::inline_candidate(composition, *i));
        match next {
            Some(i) => {
                inline.insert(i);
            }
            None => return Ok(plan),
        }
    }
}

/// Whether the frame's surfaces and chunks are cached or planned for an export (no reuse, no records).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Caching {
    /// Surfaces and chunks are cached.
    Cached,
    /// Planned for an export: nothing is reused or recorded.
    Uncached,
}

/// The state of planning one frame.
pub(crate) struct Builder<'a> {
    /// The composition being planned.
    pub(crate) comp: &'a Composition,
    /// The previous frame's records.
    pub(crate) cache: &'a CacheIndex,
    /// Window size in px.
    pub(crate) window: (u32, u32),
    /// Device px per logical px.
    pub(crate) scale: f64,
    /// Whether the GPU executes the plan (no window clears or scissors).
    pub(crate) gpu: bool,
    /// The frame number the records carry.
    pub(crate) frame: u64,
    /// Layers drawn into their parent instead of a surface.
    pub(crate) inline: HashSet<u32>,
    /// Whether this frame's surfaces and chunks are cached: no reuse, records or window damage while planning exports.
    pub(crate) caching: Caching,
    /// The targets so far.
    pub(crate) targets: Vec<TargetSpec>,
    /// The chunk tasks so far.
    pub(crate) chunks: Vec<ChunkTask<'a>>,
    /// The ops so far.
    pub(crate) ops: Vec<Op<'a>>,
    /// The chunk records so far.
    pub(crate) chunk_records: Vec<(ChunkKey, ChunkRecord)>,
    /// The layer records so far.
    pub(crate) layer_records: Vec<(LayerKey, LayerRecord)>,
    /// Layers recorded this frame.
    pub(crate) seen: HashSet<LayerKey>,
    /// Layers under a reused surface whose records are carried over.
    pub(crate) reused: HashSet<LayerKey>,
    /// Layers under (or being) a reused surface whose chunk records are carried over.
    pub(crate) reused_chunks: HashSet<LayerKey>,
    /// Window damage so far (device px, not yet clipped to the window).
    pub(crate) damage: Vec<Rect>,
    /// Window bounds of the backdrop layers, which redraw when what they cover changes.
    pub(crate) backdrops: Vec<Rect>,
    /// Surfaces reused this frame.
    pub(crate) reused_surfaces: u32,
    /// Memoised signatures by layer: (hash of the matrix and size, signature).
    pub(crate) signatures: HashMap<u32, (u64, u64)>,
}

impl<'a> Builder<'a> {
    /// A builder for one frame.
    fn new(
        comp: &'a Composition,
        cache: &'a CacheIndex,
        window: (u32, u32),
        scale: f64,
        gpu: bool,
        inline: HashSet<u32>,
    ) -> Self {
        Builder {
            comp,
            cache,
            window,
            scale,
            gpu,
            frame: cache.frame + 1,
            inline,
            caching: Caching::Cached,
            targets: vec![TargetSpec {
                size: window,
                kind: TargetKind::Temp,
            }],
            chunks: Vec::new(),
            ops: Vec::new(),
            chunk_records: Vec::new(),
            layer_records: Vec::new(),
            seen: HashSet::new(),
            reused: HashSet::new(),
            reused_chunks: HashSet::new(),
            damage: Vec::new(),
            backdrops: Vec::new(),
            reused_surfaces: 0,
            signatures: HashMap::new(),
        }
    }

    /// Whether this frame's surfaces and chunks are cached (false while planning exports).
    fn cacheable(&self) -> bool {
        self.caching == Caching::Cached
    }

    /// The layer at `index`.
    pub(crate) fn layer(
        &self,
        index: u32,
    ) -> Result<&'a kivact_types::render::CompositeLayer, PlanError> {
        self.comp
            .layers
            .get(index as usize)
            .ok_or_else(|| PlanError::new(format!("layer {index} is out of range")))
    }

    /// A new scratch target of `size`, cleared before use.
    pub(crate) fn new_temp(&mut self, size: (u32, u32)) -> TargetId {
        let id = self.push_target(size, TargetKind::Temp);
        self.ops.push(Op::Clear {
            target: id,
            rect: None,
        });
        id
    }

    /// A new target of `size` with `kind`, not cleared.
    pub(crate) fn push_target(&mut self, size: (u32, u32), kind: TargetKind) -> TargetId {
        self.targets.push(TargetSpec { size, kind });
        (self.targets.len() - 1) as TargetId
    }

    /// Plan the root layer: its items draw into the output, which has no surface.
    fn emit_root(&mut self, to_t: Matrix4) -> Result<(), PlanError> {
        let root = self.layer(0)?;
        let grid = Grid {
            origin: Point::new(0.0, 0.0),
            scale: self.scale,
            size: self.window,
        };
        let cx = ItemCtx {
            index: 0,
            layer: root,
            first: 0,
            items: &root.items,
            grid,
            dest: Dest {
                target: OUTPUT,
                to: to_t,
                size: self.window,
                cover: None,
            },
            to_win: to_t,
            cache_chunks: true,
        };
        self.emit_items(&cx)?;
        let sig = self.signature(0, &to_t, self.window)?;
        let whole = Rect::new(0.0, 0.0, f64::from(self.window.0), f64::from(self.window.1));
        self.record(root, Some(whole), sig, &to_t, &[], false);
        Ok(())
    }

    /// Plan the drawables' exports: each layer drawn into its own target.
    fn emit_exports(&mut self) -> Result<Vec<(DrawableId, TargetId)>, PlanError> {
        let comp = self.comp;
        let mut out = Vec::with_capacity(comp.exports.len());
        let saved = self.caching;
        self.caching = Caching::Uncached;
        for export in &comp.exports {
            let layer = self.layer(export.layer)?;
            let target = self.new_temp(export.size);
            let to = layer
                .transform
                .then(Matrix4::scale(export.scale, export.scale, 1.0));
            let dest = Dest {
                target,
                to,
                size: export.size,
                cover: None,
            };
            self.emit_layer(export.layer, dest, to, &[])?;
            out.push((export.sink, target));
        }
        self.caching = saved;
        Ok(out)
    }

    /// Plan the whole frame.
    fn run(mut self) -> Result<Plan<'a>, PlanError> {
        let root_to = Matrix4::scale(self.scale, self.scale, 1.0);
        if self.layer(0)?.plane == Plane::Preserve3d {
            let dest = Dest {
                target: OUTPUT,
                to: root_to,
                size: self.window,
                cover: None,
            };
            self.emit_layer(0, dest, root_to, &[])?;
        } else {
            self.emit_root(root_to)?;
        }
        let exports = self.emit_exports()?;
        Ok(self.finish(exports))
    }

    /// Finish the frame: damage, carried-over records and the output's clear and scissor.
    fn finish(mut self, exports: Vec<(DrawableId, TargetId)>) -> Plan<'a> {
        let damage = self.finish_damage();
        self.carry_reused();
        let damage_pixels = self.patch_output(&damage);
        let stats = FrameStats {
            chunks_rastered: 0,
            pixels_rastered: 0,
            surfaces_reused: self.reused_surfaces,
            damage_pixels,
            targets: self.targets.len() as u32,
        };
        Plan {
            targets: self.targets,
            chunks: self.chunks,
            ops: self.ops,
            damage,
            exports,
            chunk_records: self.chunk_records,
            layer_records: self.layer_records,
            frame: self.frame,
            stats,
        }
    }

    /// Put the output's clears in front of the ops and the scissor on its draws. Returns the damage area.
    fn patch_output(&mut self, damage: &Damage) -> u64 {
        let window_area = u64::from(self.window.0) * u64::from(self.window.1);
        if self.gpu {
            return match damage {
                Damage::Full => window_area,
                Damage::Rects(rects) => rects.iter().map(area).sum(),
            };
        }
        match damage {
            Damage::Full => {
                self.ops.insert(
                    0,
                    Op::Clear {
                        target: OUTPUT,
                        rect: None,
                    },
                );
                window_area
            }
            Damage::Rects(rects) => {
                let mut clears: Vec<Op<'a>> = rects
                    .iter()
                    .map(|r| Op::Clear {
                        target: OUTPUT,
                        rect: Some(*r),
                    })
                    .collect();
                for op in &mut self.ops {
                    if let Op::Draw(draw) = op
                        && draw.target == OUTPUT
                    {
                        draw.scissor = Scissor::Rects(rects.clone());
                    }
                }
                clears.append(&mut self.ops);
                self.ops = clears;
                rects.iter().map(area).sum()
            }
        }
    }
}

/// The area of a pixel rectangle.
pub(super) fn area(r: &PixelRect) -> u64 {
    u64::from(r.width) * u64::from(r.height)
}

/// The map from the px of a space `from` to the px of a space `to`, both given as maps from layer space.
pub(crate) fn px_map(from: &Matrix4, to: &Matrix4) -> Option<Projective> {
    Some(from.flatten().invert()?.then(to.flatten()))
}

/// The filled rectangle of a size, as a polygon in its own px.
pub(crate) fn full_polygon(size: (u32, u32)) -> Polygon {
    let (w, h) = (f64::from(size.0), f64::from(size.1));
    Polygon {
        points: vec![
            Point::new(0.0, 0.0),
            Point::new(w, 0.0),
            Point::new(w, h),
            Point::new(0.0, h),
        ],
    }
}

/// The filter's reach beyond a layer's bounds, in layer units (used by the cull and the surface size).
pub(crate) fn filter_reach(filter: &LayerFilter, bounds: Rect) -> f64 {
    match filter {
        LayerFilter::Blur { std_dev } => 3.0 * std_dev,
        LayerFilter::DropShadow {
            offset, std_dev, ..
        } => offset.x.hypot(offset.y) + 3.0 * std_dev,
        LayerFilter::ColorMatrix(_) => 0.0,
        LayerFilter::Graph(graph) => match graph.margin {
            Some(m) => m.top.max(m.right).max(m.bottom).max(m.left),
            None => 0.1 * bounds.width.max(bounds.height),
        },
        LayerFilter::Shader(shader) => shader.margin,
    }
}
