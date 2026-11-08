//! Role: Records user shader draws and filter shaders.
//!
//! A shader's pipeline comes from the context's shader cache; a program whose
//! pipeline failed draws its fallback chunk (paint) or passes its input through
//! (filter), and its failure is reported once.
use super::pass::{FrameTextures, copy, view, with_pass};
use super::pipelines::buffer;
use super::shader_cache::{IMAGE_SLOTS, make_pipeline};
use super::state::GpuContext;
use crate::ComposeError;
use crate::plan::{DrawOp, Sampling, Scissor, ShaderOp, Source};
use arvane_render::RASTER_FORMAT;
use kivact_types::effect::{ShaderProgram, ShaderUse};
use kivact_types::geometry::{Point, Size};
use kivact_types::geometry3d::Projective;
use kivact_types::pixel::{BlendMode, Pixels};
use kivact_types::render::Composite;

/// The texture of `pixels` (straight alpha) premultiplied, as the shader's images read them.
fn premultiplied_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pixels: &Pixels,
) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Arvane shader image"),
        size: wgpu::Extent3d {
            width: pixels.width,
            height: pixels.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: RASTER_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut bytes = Vec::with_capacity(pixels.rgba.len());
    for px in pixels.rgba.as_chunks::<4>().0 {
        let a = f32::from(px[3]) / 255.0;
        for c in &px[..3] {
            bytes.push((f32::from(*c) * a).round() as u8);
        }
        bytes.push(px[3]);
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * pixels.width),
            rows_per_image: Some(pixels.height),
        },
        wgpu::Extent3d {
            width: pixels.width,
            height: pixels.height,
            depth_or_array_layers: 1,
        },
    );
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// The map of target px to shader space as the module's `mat3x3` columns, padded to 16 bytes each.
fn map_columns(map: &Projective) -> Vec<f32> {
    let mut out = Vec::with_capacity(12);
    for col in 0..3 {
        for row in 0..3 {
            out.push(map.m[row][col] as f32);
        }
        out.push(0.0);
    }
    out
}

/// The coverage a shader draw multiplies in.
enum Coverage<'a> {
    /// The white coverage of a filter: nothing is multiplied in.
    White,
    /// A coverage chunk's texture.
    Texture(&'a wgpu::Texture),
}

/// What one shader draw reads besides its polygon and map.
struct Draw<'a> {
    /// The shader with its parameters and images.
    shader: &'a ShaderUse,
    /// The shader's context: its size in shader units, px per unit and the scale of its clock.
    size: (f32, f32),
    /// Device px per shader unit.
    scale: f32,
    /// The opacity multiplied in.
    opacity: f32,
    /// The content (the filter's input) and its origin and scale in shader space.
    content: Option<(&'a wgpu::Texture, (f32, f32), f32)>,
    /// The coverage the draw multiplies in.
    coverage: Coverage<'a>,
    /// Target px → shader space.
    map: [f32; 12],
}

impl GpuContext {
    /// Record one shader draw of a plan: the shader over its polygon, or its fallback chunk when the pipeline failed.
    pub(super) fn record_shader(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ShaderOp<'_>,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        let key = (op.shader.program.hash, true);
        if self.pipeline(op.shader.program.as_ref(), key).is_none() {
            return self.record_fallback(encoder, op, frame);
        }
        let coverage = self
            .chunks
            .get(&op.coverage)
            .ok_or_else(|| ComposeError::Backend("a shader's coverage chunk is missing".into()))?;
        let draw = Draw {
            shader: op.shader,
            size: (op.size.width as f32, op.size.height as f32),
            scale: op.px_per_unit as f32,
            opacity: op.opacity as f32,
            content: None,
            coverage: Coverage::Texture(coverage),
            map: map_columns(&op.map).try_into().map_err(|_| {
                ComposeError::InvalidComposition("a shader map has no 3 × 3 matrix".into())
            })?,
        };
        let target = frame.target(op.target)?;
        let vertices = fan_vertices(&op.polygon.points);
        self.draw_shader(encoder, key, &vertices, target, draw)
    }

    /// Record a filter shader of a layer: `dest` is `source` filtered. Its pipeline failed: the input passes through.
    pub(super) fn record_filter_shader(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        dest: &wgpu::Texture,
        shader: &ShaderUse,
        (px_per_unit, origin, bounds): (f64, Point, Size),
    ) -> Result<(), ComposeError> {
        let key = (shader.program.hash, false);
        if self.pipeline(shader.program.as_ref(), key).is_none() {
            copy(encoder, source, dest);
            return Ok(());
        }
        let (w, h) = (dest.width() as f32, dest.height() as f32);
        let ppu = px_per_unit as f32;
        // Target px to layer space: the origin is at px (0, 0), one layer unit is ppu px.
        // The columns of the module's mat3x3, each padded to 16 bytes.
        let map = [
            1.0 / ppu,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / ppu,
            0.0,
            0.0,
            origin.x as f32,
            origin.y as f32,
            1.0,
            0.0,
        ];
        let draw = Draw {
            shader,
            size: (bounds.width as f32, bounds.height as f32),
            scale: ppu,
            opacity: 1.0,
            content: Some((source, (origin.x as f32, origin.y as f32), ppu)),
            coverage: Coverage::White,
            map,
        };
        let full = [(0.0, 0.0), (w, 0.0), (0.0, h), (0.0, h), (w, 0.0), (w, h)];
        let vertices: Vec<f32> = full.iter().flat_map(|&(x, y)| [x, y]).collect();
        self.draw_shader(encoder, key, &vertices, dest, draw)
    }

    /// The pipeline of `program` for `key`, made on first use. None when it failed (its failure is reported once).
    fn pipeline(
        &mut self,
        program: &ShaderProgram,
        key: (u64, bool),
    ) -> Option<&wgpu::RenderPipeline> {
        let shaders = &mut self.shaders;
        if !shaders.pipelines.contains_key(&key) {
            let made = match make_pipeline(&self.device, &shaders.layout, program, key.1) {
                Ok(pipeline) => Some(pipeline),
                Err(message) => {
                    self.shader_failures.push((program.hash, message));
                    None
                }
            };
            shaders.pipelines.insert(key, made);
        }
        shaders.pipelines.get(&key).and_then(Option::as_ref)
    }

    /// Record the fallback chunk of a shader op, as the CPU draws it.
    fn record_fallback(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ShaderOp<'_>,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        let fallback = DrawOp {
            target: op.target,
            source: Source::Chunk(op.fallback),
            polygon: op.polygon.clone(),
            split_edges: Vec::new(),
            map: op.fallback_map,
            opacity: op.opacity,
            blend: BlendMode::Normal,
            composite: Composite::SourceOver,
            sampling: Sampling::Smooth,
            coverage: Some(op.coverage),
            scissor: Scissor::Off,
        };
        self.record_draw(encoder, &fallback, frame)
    }

    /// Draw `vertices` (triangles) of `target` with the pipeline of `key`, reading `draw`.
    fn draw_shader(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        key: (u64, bool),
        vertices: &[f32],
        target: &wgpu::Texture,
        draw: Draw<'_>,
    ) -> Result<(), ComposeError> {
        let Some(pipeline) = self.shaders.pipelines.get(&key).and_then(Option::as_ref) else {
            return Err(ComposeError::Backend(
                "a shader pipeline is missing for a draw".into(),
            ));
        };
        let device = &self.device;
        let queue = &self.queue;
        let context = [
            draw.size.0,
            draw.size.1,
            draw.scale,
            draw.shader.time as f32,
        ];
        let context_bytes: Vec<u8> = context.iter().flat_map(|v| v.to_le_bytes()).collect();
        let context_buffer = buffer(device, queue, wgpu::BufferUsages::UNIFORM, &context_bytes);
        let params = buffer(
            device,
            queue,
            wgpu::BufferUsages::UNIFORM,
            &draw.shader.params,
        );
        let images = self.shader_images(&draw);
        let content_view = draw.content.map(|(texture, _, _)| view(texture));
        let content_ref = content_view.as_ref().unwrap_or(&self.pipelines.dummy);
        let coverage_view = match draw.coverage {
            Coverage::White => None,
            Coverage::Texture(texture) => Some(view(texture)),
        };
        let coverage_ref = coverage_view.as_ref().unwrap_or(&self.shaders.white);
        let draw_bytes = draw_uniform_bytes(&draw, target);
        let draw_buffer = buffer(device, queue, wgpu::BufferUsages::UNIFORM, &draw_bytes);
        let vertex_bytes: Vec<u8> = vertices.iter().flat_map(|v| v.to_le_bytes()).collect();
        let vertex_buffer = buffer(device, queue, wgpu::BufferUsages::VERTEX, &vertex_bytes);
        let group = self.shader_group(
            [&context_buffer, &params, &draw_buffer],
            &images,
            content_ref,
            coverage_ref,
        );
        let out = view(target);
        let count = (vertices.len() / 2) as u32;
        with_pass(encoder, &out, wgpu::LoadOp::Load, |pass| {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..count, 0..1);
        });
        Ok(())
    }

    /// The shader's image slots as views: the dummy view stands for an empty slot.
    fn shader_images(&self, draw: &Draw<'_>) -> Vec<wgpu::TextureView> {
        let mut images: Vec<wgpu::TextureView> = Vec::with_capacity(IMAGE_SLOTS);
        for slot in &draw.shader.images {
            images.push(match slot {
                Some(pixels) => premultiplied_texture(&self.device, &self.queue, pixels),
                None => self.pipelines.dummy.clone(),
            });
        }
        images
    }

    /// The bind group of a shader draw: the uniforms `[context, params, draw]`, the sampler, the image slots, the content and the coverage.
    fn shader_group(
        &self,
        uniforms: [&wgpu::Buffer; 3],
        images: &[wgpu::TextureView],
        content: &wgpu::TextureView,
        coverage: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let [context, params, draw] = uniforms;
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arvane shader"),
            layout: &self.shaders.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: context.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.shaders.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&images[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&images[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&images[2]),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&images[3]),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(content),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(coverage),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: draw.as_entire_binding(),
                },
            ],
        })
    }
}

/// The bytes of a shader draw's uniform block: the map, the target size, the content's origin and scale, the opacity and padding.
fn draw_uniform_bytes(draw: &Draw<'_>, target: &wgpu::Texture) -> Vec<u8> {
    let (tw, th) = (target.width() as f32, target.height() as f32);
    let (origin, content_scale) = match draw.content {
        Some((_, origin, scale)) => (origin, scale),
        None => ((0.0, 0.0), 1.0),
    };
    let mut draw_bytes: Vec<u8> = Vec::with_capacity(80);
    for v in draw.map {
        draw_bytes.extend_from_slice(&v.to_le_bytes());
    }
    for v in [
        tw,
        th,
        origin.0,
        origin.1,
        content_scale,
        draw.opacity,
        0.0,
        0.0,
    ] {
        draw_bytes.extend_from_slice(&v.to_le_bytes());
    }
    draw_bytes
}

/// The triangle list of the convex polygon `points` as a fan from its first vertex, as x, y pairs.
fn fan_vertices(points: &[Point]) -> Vec<f32> {
    let mut out = Vec::new();
    if points.len() < 3 {
        return out;
    }
    for i in 1..points.len() - 1 {
        for p in [points[0], points[i], points[i + 1]] {
            out.push(p.x as f32);
            out.push(p.y as f32);
        }
    }
    out
}
