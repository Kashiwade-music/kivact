//! Role: The planning state of one frame and the entry that plans it.

use std::collections::{HashMap, HashSet};

use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect};
use kivact_types::render::{Composition, FrameStats, LayerKey, Plane};

use super::frame::{Behind, Dest, ItemCtx, area, inline_candidate};
use super::ops::{
    Caching, ChunkTask, Damage, Grid, INLINE_TARGET_LIMIT, OUTPUT, Op, Plan, PlanError,
    PlannedExport, Scissor, TargetId, TargetKind, TargetSpec,
};
use super::validate;
use crate::cache::{CacheIndex, ChunkKey, ChunkRecord, LayerRecord};

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
            .find(|i| !inline.contains(i) && inline_candidate(composition, *i));
        match next {
            Some(i) => {
                inline.insert(i);
            }
            None => return Ok(plan),
        }
    }
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
    /// The content being drawn into, outermost first: what a backdrop reads (CONTRACT-BACKDROP-ROOT).
    pub(crate) behind: Vec<Behind>,
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
            behind: Vec::new(),
            reused_surfaces: 0,
            signatures: HashMap::new(),
        }
    }

    /// Whether this frame's surfaces and chunks are cached (false while planning exports).
    pub(super) fn cacheable(&self) -> bool {
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
    fn emit_exports(&mut self) -> Result<Vec<PlannedExport>, PlanError> {
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
            let signature = self.signature(export.layer, &to, export.size)?;
            out.push(PlannedExport {
                sink: export.sink,
                target,
                signature,
            });
        }
        self.caching = saved;
        Ok(out)
    }

    /// Plan the whole frame.
    fn run(mut self) -> Result<Plan<'a>, PlanError> {
        let root_to = Matrix4::scale(self.scale, self.scale, 1.0);
        self.behind.push(Behind {
            target: OUTPUT,
            to: root_to,
            to_win: root_to,
            root: true,
        });
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
        self.behind.pop();
        let exports = self.emit_exports()?;
        Ok(self.finish(exports))
    }

    /// Finish the frame: damage, carried-over records and the output's clear and scissor.
    fn finish(mut self, exports: Vec<PlannedExport>) -> Plan<'a> {
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
