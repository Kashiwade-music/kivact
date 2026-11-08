//! Role: Items of a layer drawn into a destination: rasters, externals, shaders, clips.

use std::borrow::Cow;

use kivact_types::Matrix4;
use kivact_types::geometry::Size;
use kivact_types::pixel::BlendMode;
use kivact_types::render::{Composite, ExternalItem, LayerItem, RenderScene, ShaderItem};
use kivact_types::scene::Shape;

use super::builder::Builder;
use super::frame::{Dest, ItemCtx, coverage_scene, px_map, raster_ordinal, wrapped};
use super::ops::{DrawOp, Op, PlanError, Sampling, Scissor, ShaderOp, Source};
use crate::cache::{ChunkKey, ChunkRole};

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
        let (Some(polygon), Some(map), Some(fallback_map)) = (
            h.map_rect(shader.rect),
            h.invert(),
            px_map(&cx.dest.to, &cx.grid.to_px()),
        ) else {
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
            fallback_map,
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
        if child.backface == kivact_types::paint::BackfaceVisibility::Hidden
            && child.transform.shows_back()
        {
            return Ok(());
        }
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
