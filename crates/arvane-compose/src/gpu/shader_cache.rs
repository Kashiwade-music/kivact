//! Role: The user shaders' layouts, samplers and pipeline cache.
//!
//! Passive state of the GPU context, plus the constructors of its pipelines.
//! The draws that use them live in `shader`.

use std::collections::HashMap;

use arvane_render::RASTER_FORMAT;
use kivact_types::effect::ShaderProgram;

/// The images of a shader (slots 0..=3).
pub(super) const IMAGE_SLOTS: usize = 4;

/// The user shaders' pipelines and what their draws share.
pub(crate) struct Shaders {
    /// The layout of the bindings 0..=9 (design §11.3.3).
    pub(super) layout: wgpu::BindGroupLayout,
    /// The linear, clamp-to-edge sampler.
    pub(super) sampler: wgpu::Sampler,
    /// A 1 × 1 opaque white texture (the coverage of a filter).
    pub(super) white: wgpu::TextureView,
    /// Each program's pipeline by (hash, blended); None where the pipeline failed.
    pub(super) pipelines: HashMap<(u64, bool), Option<wgpu::RenderPipeline>>,
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
pub(super) fn make_pipeline(
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
