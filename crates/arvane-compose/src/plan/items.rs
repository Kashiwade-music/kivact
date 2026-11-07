//! Role: The items of a layer drawn into a destination: rasters, external frames, shaders, child layers and clips.

use std::borrow::Cow;

use kivact_types::Matrix4;
use kivact_types::geometry::{Rect, Size, Transform};

use kivact_types::pixel::BlendMode;
use kivact_types::render::{
    Composite, CompositeLayer, ExternalItem, Group, LayerItem, Paint, RenderCmd, RenderScene,
    ShaderItem,
};
use kivact_types::scene::Shape;

use super::{
    Builder, DrawOp, Grid, Op, PlanError, Sampling, Scissor, ShaderOp, Source, TargetId, px_map,
};
use crate::cache::{ChunkKey, ChunkRole};

/// Where a set of items draws: its target, the map from layer space to the
/// target's px, the target's size, and the clip that covers every draw.
#[derive(Clone, Copy, Debug)]
pub(super) struct Dest {
    /// The target drawn into.
    pub(super) target: TargetId,
    /// Layer space → target px.
    pub(super) to: Matrix4,
    /// The target's size in px.
    pub(super) size: (u32, u32),
    /// Coverage inherited from an inline ancestor's clip, applied to every draw.
    pub(super) cover: Option<ChunkKey>,
}

/// The layer and items being drawn.
pub(super) struct ItemCtx<'a> {
    /// Index of the layer in the composition.
    pub(super) index: u32,
    /// The layer that owns the items.
    pub(super) layer: &'a CompositeLayer,
    /// Position of `items[0]` in `layer.items`.
    pub(super) first: usize,
    /// The items to draw, in order.
    pub(super) items: &'a [LayerItem],
    /// The grid of the layer's chunks.
    pub(super) grid: Grid,
    /// Where the items draw.
    pub(super) dest: Dest,
    /// Layer space → window px, for damage.
    pub(super) to_win: Matrix4,
    /// Whether the rasters are cached (false for inline layers, which draw into their parent).
    pub(super) cache_chunks: bool,
}

impl<'a> Builder<'a> {
    /// Draw every item of `cx` in order.
    pub(super) fn emit_items(&mut self, cx: &ItemCtx<'a>) -> Result<(), PlanError> {
        for (offset, item) in cx.items.iter().enumerate() {
            let at = cx.first + offset;
            match item {
                LayerItem::Raster(scene) => self.emit_raster(cx, at, scene)?,
                LayerItem::External(external) => self.emit_external(cx, at, external)?,
                LayerItem::Shader(shader) => self.emit_shader(cx, at, shader)?,
                LayerItem::Layer { index, clips } => self.emit_child(cx, at, *index, clips)?,
            }
        }
        Ok(())
    }

    /// Raster a Vello scene into its chunk and draw the chunk into the destination.
    fn emit_raster(
        &mut self,
        cx: &ItemCtx<'a>,
        at: usize,
        scene: &'a RenderScene,
    ) -> Result<(), PlanError> {
        let key = ChunkKey {
            layer: cx.layer.key,
            role: ChunkRole::Content(raster_ordinal(cx.layer, at)),
        };
        self.chunk(
            key,
            Cow::Borrowed(scene),
            cx.grid,
            &cx.to_win,
            cx.cache_chunks,
        )?;
        let (Some(polygon), Some(map)) = (
            cx.dest.to.flatten().map_rect(cx.grid.rect()),
            px_map(&cx.dest.to, &cx.grid.to_px()),
        ) else {
            return Ok(());
        };
        self.ops.push(Op::Draw(DrawOp {
            target: cx.dest.target,
            source: Source::Chunk(key),
            polygon,
            split_edges: Vec::new(),
            map,
            opacity: 1.0,
            blend: BlendMode::Normal,
            composite: Composite::SourceOver,
            sampling: Sampling::Smooth,
            coverage: cx.dest.cover,
            scissor: Scissor::Off,
        }));
        Ok(())
    }

    /// Draw an external frame, clipped by its clips.
    fn emit_external(
        &mut self,
        cx: &ItemCtx<'a>,
        at: usize,
        external: &'a ExternalItem,
    ) -> Result<(), PlanError> {
        let coverage = if external.clips.is_empty() {
            cx.dest.cover
        } else {
            Some(self.clip_key(cx, at, &external.clips)?)
        };
        let h = Matrix4::from_affine(external.transform)
            .then(cx.dest.to)
            .flatten();
        let (Some(polygon), Some(map)) = (h.map_rect(external.rect), h.invert()) else {
            return Ok(());
        };
        self.ops.push(Op::Draw(DrawOp {
            target: cx.dest.target,
            source: Source::External {
                frame: external.frame,
                cpu: external.cpu.as_ref(),
                source: external.source,
            },
            polygon,
            split_edges: Vec::new(),
            map,
            opacity: external.opacity,
            blend: BlendMode::Normal,
            composite: Composite::SourceOver,
            sampling: Sampling::nearest(external.pixelated),
            coverage,
            scissor: Scissor::Off,
        }));
        Ok(())
    }

    /// Plan a shader: its coverage and fallback chunks, and the shader op.
    fn emit_shader(
        &mut self,
        cx: &ItemCtx<'a>,
        at: usize,
        shader: &'a ShaderItem,
    ) -> Result<(), PlanError> {
        let coverage = ChunkKey {
            layer: cx.layer.key,
            role: ChunkRole::Coverage(at as u32),
        };
        let fallback = ChunkKey {
            layer: cx.layer.key,
            role: ChunkRole::Fallback(at as u32),
        };
        self.chunk(
            coverage,
            Cow::Owned(wrapped(&shader.coverage, shader.transform)),
            cx.grid,
            &cx.to_win,
            cx.cache_chunks,
        )?;
        self.chunk(
            fallback,
            Cow::Owned(wrapped(&shader.fallback, shader.transform)),
            cx.grid,
            &cx.to_win,
            cx.cache_chunks,
        )?;
        let h = Matrix4::from_affine(shader.transform)
            .then(cx.dest.to)
            .flatten();
        let (Some(polygon), Some(map)) = (h.map_rect(shader.rect), h.invert()) else {
            return Ok(());
        };
        self.ops.push(Op::Shader(ShaderOp {
            target: cx.dest.target,
            shader: &shader.shader,
            polygon,
            map,
            size: Size::new(shader.rect.width, shader.rect.height),
            px_per_unit: h.max_scale(shader.rect),
            coverage,
            fallback,
            input: None,
            opacity: shader.opacity,
        }));
        Ok(())
    }

    /// Plan a child layer drawn into the destination, under its clips.
    fn emit_child(
        &mut self,
        cx: &ItemCtx<'a>,
        at: usize,
        index: u32,
        clips: &'a [Shape],
    ) -> Result<(), PlanError> {
        let child = self.layer(index)?;
        let cover = if clips.is_empty() {
            cx.dest.cover
        } else {
            Some(self.clip_key(cx, at, clips)?)
        };
        let to_child = child.transform.then(cx.dest.to);
        let win_child = child.transform.then(cx.to_win);
        let dest = Dest {
            target: cx.dest.target,
            to: to_child,
            size: cx.dest.size,
            cover,
        };
        self.emit_layer(index, dest, win_child, clips)
    }

    /// The coverage chunk of the clips of item `at` of the layer, on the layer's grid.
    fn clip_key(
        &mut self,
        cx: &ItemCtx<'a>,
        at: usize,
        clips: &[Shape],
    ) -> Result<ChunkKey, PlanError> {
        let key = ChunkKey {
            layer: cx.layer.key,
            role: ChunkRole::Clip(at as u32),
        };
        let scene = coverage_scene(clips, cx.grid.rect());
        self.chunk(key, Cow::Owned(scene), cx.grid, &cx.to_win, cx.cache_chunks)?;
        Ok(key)
    }
}

/// The number of Raster items before position `at` of the layer (the chunk ordinal of a raster).
pub(super) fn raster_ordinal(layer: &CompositeLayer, at: usize) -> u32 {
    layer.items[..at]
        .iter()
        .filter(|item| matches!(item, LayerItem::Raster(_)))
        .count() as u32
}

/// A white coverage scene: the rectangle `rect` clipped by every shape of `clips`.
pub(super) fn coverage_scene(clips: &[Shape], rect: Rect) -> RenderScene {
    let white = kivact_types::Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let mut commands = Vec::with_capacity(clips.len() * 2 + 3);
    for clip in clips {
        commands.push(RenderCmd::PushGroup(Group {
            clip: Some(clip.clone()),
            opacity: 1.0,
            blend: BlendMode::Normal,
            blur: None,
        }));
    }
    commands.push(RenderCmd::FillShape {
        shape: Shape::Rect(rect),
        paint: Paint::Solid(white),
    });
    commands.extend(clips.iter().map(|_| RenderCmd::PopGroup));
    RenderScene { commands }
}

/// A scene placed by `t` in its owner's space: the commands wrapped in a transform.
pub(super) fn wrapped(scene: &RenderScene, t: Transform) -> RenderScene {
    let mut commands = Vec::with_capacity(scene.commands.len() + 2);
    commands.push(RenderCmd::PushTransform(t));
    commands.extend(scene.commands.iter().cloned());
    commands.push(RenderCmd::PopTransform);
    RenderScene { commands }
}
