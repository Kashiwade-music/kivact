//! Role: The plan's data types: targets, chunk tasks, ops, damage and errors.
//!
//! The plan knows no backend. A CPU or GPU executor draws these ops in order.

use std::borrow::Cow;

use kivact_types::Matrix4;
use kivact_types::effect::ShaderUse;
use kivact_types::geometry::{Point, Rect, Size};
use kivact_types::geometry3d::{Polygon, Projective};
use kivact_types::gpu::ExternalFrame;
use kivact_types::ids::DrawableId;
use kivact_types::paint::{MaskComposite, MaskMode};
use kivact_types::pixel::{BlendMode, Pixels};
use kivact_types::render::{Composite, FrameStats, LayerFilter, LayerKey, PixelRect, RenderScene};

use crate::cache::{ChunkKey, ChunkRecord, LayerRecord};

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
    #[cfg_attr(
        not(feature = "gpu"),
        expect(dead_code, reason = "read only by the GPU executor")
    )]
    pub(crate) shader: &'a ShaderUse,
    /// Destination polygon in target px.
    pub(crate) polygon: Polygon,
    /// Target px → shader space.
    #[cfg_attr(
        not(feature = "gpu"),
        expect(dead_code, reason = "read only by the GPU executor")
    )]
    pub(crate) map: Projective,
    /// The shader's coordinate box size in logical px.
    #[cfg_attr(
        not(feature = "gpu"),
        expect(dead_code, reason = "read only by the GPU executor")
    )]
    pub(crate) size: Size,
    /// Shader px per logical px.
    #[cfg_attr(
        not(feature = "gpu"),
        expect(dead_code, reason = "read only by the GPU executor")
    )]
    pub(crate) px_per_unit: f64,
    /// The coverage chunk.
    pub(crate) coverage: ChunkKey,
    /// The fallback chunk the CPU draws instead.
    pub(crate) fallback: ChunkKey,
    /// Target px → fallback chunk px. The chunk already holds the item's transform,
    /// so this is the layer-to-px map of the chunk's grid composed with the target's.
    pub(crate) fallback_map: Projective,
    /// Alpha multiplier.
    pub(crate) opacity: f64,
    /// Pixels of the target outside these rectangles stay as they are (window damage).
    pub(crate) scissor: Scissor,
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
        /// The layer's bounds in layer units: the size of a filter shader's box.
        #[cfg_attr(
            not(feature = "gpu"),
            expect(dead_code, reason = "read only by the GPU executor")
        )]
        bounds: Size,
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

/// One export of the frame: the target that holds its frame and the content signature of its layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlannedExport {
    /// The drawable that receives the frame.
    pub(crate) sink: DrawableId,
    /// The target holding the frame.
    pub(crate) target: TargetId,
    /// The layer's signature at the export's transform and size.
    pub(crate) signature: u64,
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
    pub(crate) exports: Vec<PlannedExport>,
    /// Chunk records to store after a successful frame.
    pub(crate) chunk_records: Vec<(ChunkKey, ChunkRecord)>,
    /// Layer records to store after a successful frame.
    pub(crate) layer_records: Vec<(LayerKey, LayerRecord)>,
    /// The frame number the records carry.
    pub(crate) frame: u64,
    /// The statistics this plan decides (the executor adds the raster counts).
    pub(crate) stats: FrameStats,
}

/// Whether the frame's surfaces and chunks are cached or planned for an export (no reuse, no records).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Caching {
    /// Surfaces and chunks are cached.
    Cached,
    /// Planned for an export: nothing is reused or recorded.
    Uncached,
}
