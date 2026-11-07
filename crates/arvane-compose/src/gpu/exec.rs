//! Role: Runs one frame's plan on the GPU: its targets and chunks, the operations in order, and the exports.

use super::external::{self, ExternalPass};
use super::pipelines::buffer;
use super::state::GpuContext;
use super::targets;
use crate::ComposeError;
use crate::cache::{CacheIndex, ChunkKey, ChunkRecord, LayerRecord};
use crate::plan::{self, ChunkTask, Op, Plan, Redraw, TargetKind, TargetSpec};
use arvane_render::Canvas;
use kivact_types::geometry::Point;
use kivact_types::pixel::Pixels;
use kivact_types::render::{
    Composition, ExportFrame, ExportedFrame, FrameReport, FrameStats, LayerKey, RenderScene,
};

/// The textures one frame draws with: its targets, by target id, and the external frames it may read.
pub(super) struct FrameTextures<'a> {
    pub(super) targets: Vec<wgpu::Texture>,
    pub(super) external: &'a external::Frame,
}

impl FrameTextures<'_> {
    /// The texture of target `id`.
    pub(super) fn target(&self, id: u32) -> Result<&wgpu::Texture, ComposeError> {
        self.targets
            .get(id as usize)
            .ok_or_else(|| ComposeError::InvalidComposition(format!("target {id} is out of range")))
    }
}

/// The records and output of one planned frame, before the cache keeps them.
pub(super) struct Records {
    pub(super) frame: u64,
    pub(super) chunks: Vec<(ChunkKey, ChunkRecord)>,
    pub(super) layers: Vec<(LayerKey, LayerRecord)>,
}

/// One drawn frame: its output (the window's size) and what it produced.
pub(super) struct Drawn {
    pub(super) output: wgpu::Texture,
    pub(super) report: FrameReport,
}

/// A view of all of `texture`.
pub(super) fn view(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// The size of `texture` in px, as floats.
pub(super) fn size_f(texture: &wgpu::Texture) -> (f32, f32) {
    (texture.width() as f32, texture.height() as f32)
}

/// Run `draw` in one render pass into `view`, loaded as `load`.
pub(super) fn with_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    draw: impl FnOnce(&mut wgpu::RenderPass<'_>),
) {
    let attachment = wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load,
            store: wgpu::StoreOp::Store,
        },
    };
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Arvane compositor pass"),
        color_attachments: &[Some(attachment)],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    draw(&mut pass);
}

/// The transparent colour that clears a target.
pub(super) const CLEAR: wgpu::LoadOp<wgpu::Color> = wgpu::LoadOp::Clear(wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
});

impl GpuContext {
    /// Plan and draw one frame of `composition` at `size` px and `scale`.
    /// The output stays in the returned texture until the caller gives it back.
    pub(super) fn draw_frame(
        &mut self,
        composition: &Composition,
        size: (u32, u32),
        scale: f64,
    ) -> Result<Drawn, ComposeError> {
        self.check()?;
        let cache = std::mem::take(&mut self.cache);
        match self.draw_planned(composition, size, scale, &cache) {
            Ok((drawn, records)) => {
                let mut cache = cache;
                cache.store(records.frame, records.chunks, records.layers);
                self.cache = cache;
                self.prune();
                Ok(drawn)
            }
            Err(error) => {
                self.release_caches();
                Err(error)
            }
        }
    }

    /// Drop the chunk and surface textures no record of the cache names.
    fn prune(&mut self) {
        let cache = &self.cache;
        self.chunks.retain(|key, _| cache.chunks.contains_key(key));
        self.kept.retain(|key, _| cache.layers.contains_key(key));
    }

    /// Plan one frame against `cache`, draw it, and return the output with the records to store.
    fn draw_planned(
        &mut self,
        composition: &Composition,
        size: (u32, u32),
        scale: f64,
        cache: &CacheIndex,
    ) -> Result<(Drawn, Records), ComposeError> {
        let plan = plan::build(composition, size, scale, cache, true)
            .map_err(|error| ComposeError::InvalidComposition(error.message))?;
        if plan.targets.len() > plan::INLINE_TARGET_LIMIT {
            return Err(ComposeError::GpuLimit(format!(
                "{} targets in one frame",
                plan.targets.len()
            )));
        }
        let limit = self.device.limits().max_texture_dimension_2d;
        if plan
            .targets
            .iter()
            .any(|t| t.size.0 > limit || t.size.1 > limit)
        {
            return Err(ComposeError::GpuLimit(format!(
                "a target is larger than {limit} px"
            )));
        }
        let Plan {
            targets,
            chunks,
            ops,
            exports,
            chunk_records,
            layer_records,
            frame,
            mut stats,
            ..
        } = plan;
        let mut textures = self.allocate(&targets)?;
        for task in &chunks {
            self.raster_chunk(task, &mut stats)?;
        }
        let frames = external_frames(&ops);
        let mut external = std::mem::take(&mut self.external);
        let result = self.draw_ops(
            &mut external,
            &frames,
            &ops,
            &mut textures,
            &exports,
            composition,
        );
        self.external = external;
        let (exported, missed) = result?;
        let output = self.finish_targets(&targets, textures);
        let report = FrameReport {
            exports: exported,
            missed,
            shader_failures: Vec::new(),
            pending_exports: Vec::new(),
            stats,
        };
        let records = Records {
            frame,
            chunks: chunk_records,
            layers: layer_records,
        };
        Ok((Drawn { output, report }, records))
    }

    /// Acquire the external frames, draw the operations, submit, and read the exports back.
    /// Returns the exported frames and the external frames that were missed.
    fn draw_ops(
        &mut self,
        external: &mut external::External,
        frames: &[kivact_types::ExternalFrame],
        ops: &[Op<'_>],
        textures: &mut Vec<wgpu::Texture>,
        exports: &[(kivact_types::ids::DrawableId, u32)],
        composition: &Composition,
    ) -> Result<(Vec<ExportedFrame>, Vec<kivact_types::ExternalFrame>), ComposeError> {
        let pass = ExternalPass::begin(external, frames)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane compositor frame"),
            });
        let frame = FrameTextures {
            targets: std::mem::take(textures),
            external: pass.frame(),
        };
        let recorded = self.record_ops(&mut encoder, ops, &frame);
        let FrameTextures { targets, .. } = frame;
        *textures = targets;
        let submitted = pass.submit();
        self.queue.submit([encoder.finish()]);
        submitted.finish()?;
        recorded?;
        let exported = self.read_exports(textures, exports, composition)?;
        self.check()?;
        Ok((exported, external.take_missed()))
    }

    /// Record every operation of the plan, in order.
    fn record_ops(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        ops: &[Op<'_>],
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        // The output starts transparent; the plan clears only the targets it draws into.
        with_pass(encoder, &view(frame.target(0)?), CLEAR, |_| {});
        for op in ops {
            match op {
                Op::Clear { target, rect } => self.record_clear(encoder, *target, *rect, frame)?,
                Op::Draw(draw) => self.record_draw(encoder, draw, frame)?,
                Op::Filter {
                    input,
                    output,
                    filter,
                    px_per_unit,
                    ..
                } => self.record_filter(encoder, *input, *output, filter, *px_per_unit, frame)?,
                Op::Mask { target, masks } => self.record_mask(encoder, *target, masks, frame)?,
                Op::Clip { target, coverage } => {
                    self.record_clip(encoder, *target, *coverage, frame)?
                }
                // The user shader is not drawn yet: its fallback chunk needs the
                // item-to-layer transform, which the plan does not carry.
                Op::Shader(_) => {}
            }
        }
        Ok(())
    }

    /// Clear a target, or a rectangle of it, to transparent.
    fn record_clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: u32,
        rect: Option<kivact_types::render::PixelRect>,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        let view = view(frame.target(target)?);
        match rect {
            None => with_pass(encoder, &view, CLEAR, |_| {}),
            Some(r) => with_pass(encoder, &view, wgpu::LoadOp::Load, |pass| {
                pass.set_pipeline(&self.pipelines.clear);
                pass.set_scissor_rect(r.x, r.y, r.width, r.height);
                pass.draw(0..3, 0..1);
            }),
        }
        Ok(())
    }

    /// Make the textures of the plan's targets: pooled, or kept from earlier frames.
    fn allocate(&mut self, specs: &[TargetSpec]) -> Result<Vec<wgpu::Texture>, ComposeError> {
        let mut out = Vec::with_capacity(specs.len());
        for spec in specs {
            let texture = match spec.kind {
                TargetKind::Temp => self.pool.take(&self.device, spec.size),
                TargetKind::Kept { key, reuse, .. } => match self.kept.remove(&key) {
                    Some(texture) if (texture.width(), texture.height()) == spec.size => texture,
                    Some(texture) => {
                        if reuse {
                            return Err(ComposeError::Backend(
                                "a reused surface changed size".into(),
                            ));
                        }
                        self.pool.give(texture);
                        targets::target(&self.device, spec.size)
                    }
                    None if reuse => {
                        return Err(ComposeError::Backend("a reused surface is missing".into()));
                    }
                    None => targets::target(&self.device, spec.size),
                },
            };
            out.push(texture);
        }
        Ok(out)
    }

    /// Give the frame's textures back: kept surfaces to their layers, the rest to the pool.
    /// Returns the output texture (target 0).
    fn finish_targets(
        &mut self,
        specs: &[TargetSpec],
        textures: Vec<wgpu::Texture>,
    ) -> wgpu::Texture {
        let mut output = None;
        for (index, (spec, texture)) in specs.iter().zip(textures).enumerate() {
            match spec.kind {
                TargetKind::Kept { key, .. } => {
                    if let Some(old) = self.kept.insert(key, texture) {
                        self.pool.give(old);
                    }
                }
                TargetKind::Temp if index == 0 => output = Some(texture),
                TargetKind::Temp => self.pool.give(texture),
            }
        }
        output.unwrap_or_else(|| targets::target(&self.device, (0, 0)))
    }

    /// Read the exported targets back as straight-alpha pixels, one per request.
    fn read_exports(
        &self,
        textures: &[wgpu::Texture],
        exports: &[(kivact_types::ids::DrawableId, u32)],
        composition: &Composition,
    ) -> Result<Vec<ExportedFrame>, ComposeError> {
        let mut out = Vec::with_capacity(exports.len());
        for ((sink, target), export) in exports.iter().zip(&composition.exports) {
            let texture = textures.get(*target as usize).ok_or_else(|| {
                ComposeError::InvalidComposition("an export target is missing".into())
            })?;
            let mut rgba = self.read_rgba(texture)?;
            unpremultiply(&mut rgba);
            let pixels = Pixels::new(texture.width(), texture.height(), rgba);
            out.push(ExportedFrame {
                sink: *sink,
                generation: export.generation,
                frame: ExportFrame::Pixels(pixels),
            });
        }
        Ok(out)
    }

    /// Raster one chunk into its texture (all of it, or the damaged rectangles).
    fn raster_chunk(
        &mut self,
        task: &ChunkTask<'_>,
        stats: &mut FrameStats,
    ) -> Result<(), ComposeError> {
        let (w, h) = task.size;
        let existing = self.chunks.remove(&task.key);
        let (texture, redraw) = match existing {
            Some(texture) if (texture.width(), texture.height()) == (w, h) => {
                (texture, task.redraw.clone())
            }
            _ => (targets::target(&self.device, (w, h)), Redraw::All),
        };
        match redraw {
            Redraw::All => {
                self.raster_into(&task.scene, task.origin, task.scale, (w, h), &texture)?;
                stats.pixels_rastered += u64::from(w) * u64::from(h);
            }
            Redraw::Rects(rects) => {
                for r in rects {
                    let origin = Point::new(
                        task.origin.x + f64::from(r.x) / task.scale,
                        task.origin.y + f64::from(r.y) / task.scale,
                    );
                    let part = targets::target(&self.device, (r.width, r.height));
                    self.raster_into(&task.scene, origin, task.scale, (r.width, r.height), &part)?;
                    let mut encoder =
                        self.device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Arvane chunk patch"),
                            });
                    encoder.copy_texture_to_texture(
                        part.as_image_copy(),
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: r.x,
                                y: r.y,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        part.size(),
                    );
                    self.queue.submit([encoder.finish()]);
                    stats.pixels_rastered += u64::from(r.width) * u64::from(r.height);
                }
            }
        }
        stats.chunks_rastered += 1;
        self.chunks.insert(task.key, texture);
        self.check()
    }

    /// Raster `scene` into `texture` through its own submission.
    fn raster_into(
        &mut self,
        scene: &RenderScene,
        origin: Point,
        scale: f64,
        size: (u32, u32),
        texture: &wgpu::Texture,
    ) -> Result<(), ComposeError> {
        let (w, h) = (
            u16::try_from(size.0).map_err(|_| ComposeError::InvalidSize)?,
            u16::try_from(size.1).map_err(|_| ComposeError::InvalidSize)?,
        );
        let canvas = Canvas {
            origin,
            scale,
            size: (w, h),
        };
        let view = view(texture);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Arvane chunk raster"),
            });
        self.raster.raster(
            &self.device,
            &self.queue,
            &mut encoder,
            scene,
            canvas,
            &view,
        )?;
        self.queue.submit([encoder.finish()]);
        Ok(())
    }
}

/// The external frames the operations read, in order.
fn external_frames(ops: &[Op<'_>]) -> Vec<kivact_types::ExternalFrame> {
    ops.iter()
        .filter_map(|op| match op {
            Op::Draw(draw) => match &draw.source {
                plan::Source::External { frame, .. } => Some(*frame),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Unpremultiply RGBA8 in place (straight alpha, as exported pixels are).
fn unpremultiply(rgba: &mut [u8]) {
    for px in rgba.as_chunks_mut::<4>().0 {
        let a = u32::from(px[3]);
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
        } else if a < 255 {
            for c in &mut px[..3] {
                *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
}

/// Keeps `buffer` alive for the frame (its bind groups reference it).
pub(super) fn uniform(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> wgpu::Buffer {
    buffer(device, queue, wgpu::BufferUsages::UNIFORM, bytes)
}
