//! Role: Runs the user shaders of a plan on the GPU (design §8.10, §11.3.4): one pipeline per program, cached by its hash, and the draws over their polygons. A program whose pipeline fails draws its fallback instead.

use std::collections::HashMap;

use super::exec::{FrameTextures, view, with_pass};
use super::filters::copy;
use super::pipelines::buffer;
use super::state::GpuContext;
use crate::ComposeError;
use crate::plan::{DrawOp, Sampling, Scissor, ShaderOp, Source};
use arvane_render::RASTER_FORMAT;
use kivact_types::effect::{ShaderProgram, ShaderUse};
use kivact_types::geometry::Point;
use kivact_types::geometry3d::Projective;
use kivact_types::pixel::{BlendMode, Pixels};
use kivact_types::render::Composite;

/// The images of a shader (slots 0..=3).
const IMAGE_SLOTS: usize = 4;

/// The user shaders' pipelines and what their draws share.
pub(crate) struct Shaders {
    /// The layout of the bindings 0..=9 (design §11.3.3).
    layout: wgpu::BindGroupLayout,
    /// The linear, clamp-to-edge sampler.
    sampler: wgpu::Sampler,
    /// A 1 × 1 opaque white texture (the coverage of a filter).
    white: wgpu::TextureView,
    /// Each program's pipeline by (hash, blended); None where the pipeline failed.
    pipelines: HashMap<(u64, bool), Option<wgpu::RenderPipeline>>,
}

impl Shaders {
    /// The shared resources of the user shaders on `device`.
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Arvane shader"),
            entries: &shader_entries(),
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Arvane shader sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let white = pixel_texture(device, queue, [255; 4]);
        Self {
            layout,
            sampler,
            white,
            pipelines: HashMap::new(),
        }
    }
}

/// The layout entries of the bindings 0..=9 (uniforms, sampler, textures).
fn shader_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
    let uniform = |binding: u32| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let texture = |binding: u32| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    vec![
        uniform(0),
        uniform(1),
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        texture(3),
        texture(4),
        texture(5),
        texture(6),
        texture(7),
        texture(8),
        uniform(9),
    ]
}

/// A 1 × 1 texture of `pixel` (premultiplied), as a view.
fn pixel_texture(device: &wgpu::Device, queue: &wgpu::Queue, pixel: [u8; 4]) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Arvane shader pixel"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: RASTER_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixel,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// The pipeline of `program` on `device`, or the failure message the device reported (driver errors, design §8.10).
fn make_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    program: &ShaderProgram,
    blended: bool,
) -> Result<wgpu::RenderPipeline, String> {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Arvane shader"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    // A blended paint writes its premultiplied colour over the target; a filter replaces it.
    let blend = blended.then_some(wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    });
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Arvane shader"),
        source: wgpu::ShaderSource::Wgsl((&*program.module).into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Arvane shader"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("kivact_vertex"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
            }],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("kivact_fragment"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: RASTER_FORMAT,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    match pollster::block_on(scope.pop()) {
        None => Ok(pipeline),
        Some(error) => Err(error.to_string()),
    }
}

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
    /// The coverage texture, or None for the white coverage of a filter.
    coverage: Option<&'a wgpu::Texture>,
    /// Target px → shader space.
    map: [f32; 12],
}

impl GpuContext {
    /// Record one shader draw of a plan: the shader over its polygon, or its fallback chunk when the pipeline failed (design §8.10).
    pub(super) fn record_shader(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        op: &ShaderOp<'_>,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        if op.input.is_some() {
            return Err(ComposeError::InvalidComposition(
                "a paint shader reads a target, which the plan does not produce".into(),
            ));
        }
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
            coverage: Some(coverage),
            map: map_columns(&op.map).try_into().map_err(|_| {
                ComposeError::InvalidComposition("a shader map has no 3 × 3 matrix".into())
            })?,
        };
        let target = frame.target(op.target)?;
        let vertices = fan_vertices(&op.polygon.points);
        self.draw_shader(encoder, key, &vertices, target, draw)
    }

    /// Record a filter shader of a layer: `dest` is `source` filtered (design §8.8.5). Its pipeline failed: the input passes through.
    pub(super) fn record_filter_shader(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        dest: &wgpu::Texture,
        shader: &ShaderUse,
        px_per_unit: f64,
        origin: Point,
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
            size: (w / ppu, h / ppu),
            scale: ppu,
            opacity: 1.0,
            content: Some((source, (origin.x as f32, origin.y as f32), ppu)),
            coverage: None,
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
        let mut images: Vec<wgpu::TextureView> = Vec::with_capacity(IMAGE_SLOTS);
        for slot in &draw.shader.images {
            images.push(match slot {
                Some(pixels) => premultiplied_texture(device, queue, pixels),
                None => self.pipelines.dummy.clone(),
            });
        }
        let content_view = draw.content.map(|(texture, _, _)| view(texture));
        let content_ref = content_view.as_ref().unwrap_or(&self.pipelines.dummy);
        let coverage_view = draw.coverage.map(view);
        let coverage_ref = coverage_view.as_ref().unwrap_or(&self.shaders.white);
        let (origin, content_scale) = match draw.content {
            Some((_, origin, scale)) => (origin, scale),
            None => ((0.0, 0.0), 1.0),
        };
        let (tw, th) = (target.width() as f32, target.height() as f32);
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
        let draw_buffer = buffer(device, queue, wgpu::BufferUsages::UNIFORM, &draw_bytes);
        let vertex_bytes: Vec<u8> = vertices.iter().flat_map(|v| v.to_le_bytes()).collect();
        let vertex_buffer = buffer(device, queue, wgpu::BufferUsages::VERTEX, &vertex_bytes);
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arvane shader"),
            layout: &self.shaders.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: context_buffer.as_entire_binding(),
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
                    resource: wgpu::BindingResource::TextureView(content_ref),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(coverage_ref),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: draw_buffer.as_entire_binding(),
                },
            ],
        });
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
