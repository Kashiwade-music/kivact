//! Role: The compositor's render pipelines, bind group layouts and shared buffers, made once per device.

use arvane_render::RASTER_FORMAT;
use kivact_types::render::Composite;
use std::collections::HashMap;

/// The draw shader: the operations' sampling, coverage and blends.
const DRAW_WGSL: &str = concat!(
    include_str!("wgsl/common.wgsl"),
    include_str!("wgsl/draw.wgsl")
);
/// The filter shader: resampling, blur, shadow and colour matrix.
const FILTER_WGSL: &str = concat!(
    include_str!("wgsl/common.wgsl"),
    include_str!("wgsl/filter.wgsl")
);
/// The mask shader: one mask chunk's value folded into the accumulator.
const MASK_WGSL: &str = concat!(
    include_str!("wgsl/common.wgsl"),
    include_str!("wgsl/mask.wgsl")
);
/// The apply shader: full-target writes from one texture, and the swapchain copy.
const APPLY_WGSL: &str = concat!(
    include_str!("wgsl/common.wgsl"),
    include_str!("wgsl/apply.wgsl")
);

/// The format of the mask accumulators.
pub(crate) const ACC_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

/// The index of the blend-in-shader pipeline in [`Pipelines::draw`] (after the eleven composites).
pub(crate) const SHADER_BLEND: usize = 11;

/// The render pipelines and layouts the executor draws with.
pub(crate) struct Pipelines {
    /// One per composite (by [`composite_slot`]), then the shader blend.
    pub(crate) draw: Vec<wgpu::RenderPipeline>,
    pub(crate) draw_layout: wgpu::BindGroupLayout,
    pub(crate) filter: wgpu::RenderPipeline,
    pub(crate) filter_layout: wgpu::BindGroupLayout,
    pub(crate) mask_step: wgpu::RenderPipeline,
    pub(crate) mask_layout: wgpu::BindGroupLayout,
    /// Multiplies the target by the accumulator's value.
    pub(crate) apply_acc: wgpu::RenderPipeline,
    /// Multiplies the target by a texture's alpha (clips).
    pub(crate) apply_alpha: wgpu::RenderPipeline,
    pub(crate) apply_layout: wgpu::BindGroupLayout,
    /// Clears a scissored rectangle to transparent.
    pub(crate) clear: wgpu::RenderPipeline,
    /// The swapchain copies, by format.
    pub(crate) present: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
    pub(crate) apply_module: wgpu::ShaderModule,
    /// A 1 × 1 texture bound where an operation has no texture.
    pub(crate) dummy: wgpu::TextureView,
    /// A 1 × 1 accumulator bound where a mask has no accumulator.
    pub(crate) dummy_acc: wgpu::TextureView,
    /// A buffer bound where an operation has no storage.
    pub(crate) dummy_buffer: wgpu::Buffer,
}

/// A byte builder for uniform and storage data (little-endian, WGSL layout).
#[derive(Default)]
pub(crate) struct Words(Vec<u8>);

impl Words {
    /// Append one `f32`.
    pub(crate) fn f(&mut self, value: f32) -> &mut Self {
        self.0.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append one `u32`.
    pub(crate) fn u(&mut self, value: u32) -> &mut Self {
        self.0.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Append a `vec4<f32>`.
    pub(crate) fn v4(&mut self, v: [f32; 4]) -> &mut Self {
        for value in v {
            self.f(value);
        }
        self
    }

    /// The bytes built so far.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0
    }
}

/// The slot of a composite in [`Pipelines::draw`].
pub(crate) fn composite_slot(composite: Composite) -> usize {
    match composite {
        Composite::SourceOver => 0,
        Composite::SourceIn => 1,
        Composite::SourceOut => 2,
        Composite::SourceAtop => 3,
        Composite::DestinationOver => 4,
        Composite::DestinationIn => 5,
        Composite::DestinationOut => 6,
        Composite::DestinationAtop => 7,
        Composite::Lighter => 8,
        Composite::Copy => 9,
        Composite::Xor => 10,
    }
}

/// The source and destination factors of a Porter–Duff operator (design §8.7).
fn factors(composite: Composite) -> (wgpu::BlendFactor, wgpu::BlendFactor) {
    use wgpu::BlendFactor as F;
    match composite {
        Composite::SourceOver => (F::One, F::OneMinusSrcAlpha),
        Composite::SourceIn => (F::DstAlpha, F::Zero),
        Composite::SourceOut => (F::OneMinusDstAlpha, F::Zero),
        Composite::SourceAtop => (F::DstAlpha, F::OneMinusSrcAlpha),
        Composite::DestinationOver => (F::OneMinusDstAlpha, F::One),
        Composite::DestinationIn => (F::Zero, F::SrcAlpha),
        Composite::DestinationOut => (F::Zero, F::OneMinusSrcAlpha),
        Composite::DestinationAtop => (F::OneMinusDstAlpha, F::SrcAlpha),
        Composite::Lighter => (F::One, F::One),
        Composite::Copy => (F::One, F::Zero),
        Composite::Xor => (F::OneMinusDstAlpha, F::OneMinusSrcAlpha),
    }
}

/// A blend state with the given factors for colour and alpha.
fn blend(src: wgpu::BlendFactor, dst: wgpu::BlendFactor) -> wgpu::BlendState {
    let component = wgpu::BlendComponent {
        src_factor: src,
        dst_factor: dst,
        operation: wgpu::BlendOperation::Add,
    };
    wgpu::BlendState {
        color: component,
        alpha: component,
    }
}

/// A pipeline of `module` drawing the fragment `fs` with `vs` into `format`.
fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    (vs, fs): (&str, &str),
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Arvane compositor"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(vs),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(fs),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// A pipeline layout of the given bind group layouts.
fn layout(device: &wgpu::Device, groups: &[&wgpu::BindGroupLayout]) -> wgpu::PipelineLayout {
    let entries: Vec<Option<&wgpu::BindGroupLayout>> = groups.iter().map(|g| Some(*g)).collect();
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Arvane compositor"),
        bind_group_layouts: &entries,
        immediate_size: 0,
    })
}

/// A bind group layout entry of a float texture (read with `textureLoad` only).
pub(crate) fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

/// A bind group layout entry of a uniform buffer.
pub(crate) fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// A bind group layout entry of a read-only storage buffer.
pub(crate) fn storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// A bind group layout of `entries`.
fn group_layout(
    device: &wgpu::Device,
    entries: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Arvane compositor"),
        entries,
    })
}

/// A buffer holding `bytes` (padded to 16 bytes), with `usage`.
pub(crate) fn buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: wgpu::BufferUsages,
    bytes: &[u8],
) -> wgpu::Buffer {
    let size = bytes.len().max(16).next_multiple_of(16) as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Arvane compositor buffer"),
        size,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, bytes);
    buffer
}

/// A 1 × 1 texture of `format`, as a view.
fn dummy_view(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Arvane compositor dummy"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

/// A shader module of `source`.
fn shader(device: &wgpu::Device, source: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Arvane compositor"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

impl Pipelines {
    /// Make every pipeline the executor draws with on `device`.
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let draw_module = shader(device, DRAW_WGSL);
        let filter_module = shader(device, FILTER_WGSL);
        let mask_module = shader(device, MASK_WGSL);
        let apply_module = shader(device, APPLY_WGSL);
        let draw_layout = group_layout(
            device,
            &[
                uniform_entry(0),
                texture_entry(1),
                texture_entry(2),
                texture_entry(3),
                storage_entry(4),
            ],
        );
        let filter_layout = group_layout(
            device,
            &[
                uniform_entry(0),
                texture_entry(1),
                texture_entry(2),
                storage_entry(3),
            ],
        );
        let mask_layout = group_layout(
            device,
            &[uniform_entry(0), texture_entry(1), texture_entry(2)],
        );
        let apply_layout = group_layout(device, &[texture_entry(0)]);
        let draw_pl = layout(device, &[&draw_layout]);
        let filter_pl = layout(device, &[&filter_layout]);
        let mask_pl = layout(device, &[&mask_layout]);
        let apply_pl = layout(device, &[&apply_layout]);
        let empty_pl = layout(device, &[]);
        let mut draw = Vec::with_capacity(SHADER_BLEND + 1);
        for composite in [
            Composite::SourceOver,
            Composite::SourceIn,
            Composite::SourceOut,
            Composite::SourceAtop,
            Composite::DestinationOver,
            Composite::DestinationIn,
            Composite::DestinationOut,
            Composite::DestinationAtop,
            Composite::Lighter,
            Composite::Copy,
            Composite::Xor,
        ] {
            let (src, dst) = factors(composite);
            draw.push(pipeline(
                device,
                &draw_pl,
                &draw_module,
                ("vs_draw", "fs_draw"),
                RASTER_FORMAT,
                Some(blend(src, dst)),
            ));
        }
        draw.push(pipeline(
            device,
            &draw_pl,
            &draw_module,
            ("vs_draw", "fs_blend"),
            RASTER_FORMAT,
            None,
        ));
        let multiply = blend(wgpu::BlendFactor::Zero, wgpu::BlendFactor::Src);
        Self {
            draw,
            filter: pipeline(
                device,
                &filter_pl,
                &filter_module,
                ("vs_full", "fs_filter"),
                RASTER_FORMAT,
                None,
            ),
            mask_step: pipeline(
                device,
                &mask_pl,
                &mask_module,
                ("vs_full", "fs_mask_step"),
                ACC_FORMAT,
                None,
            ),
            apply_acc: pipeline(
                device,
                &apply_pl,
                &apply_module,
                ("vs_full", "fs_apply_acc"),
                RASTER_FORMAT,
                Some(multiply),
            ),
            apply_alpha: pipeline(
                device,
                &apply_pl,
                &apply_module,
                ("vs_full", "fs_apply_alpha"),
                RASTER_FORMAT,
                Some(multiply),
            ),
            clear: pipeline(
                device,
                &empty_pl,
                &draw_module,
                ("vs_full", "fs_clear"),
                RASTER_FORMAT,
                None,
            ),
            present: HashMap::new(),
            dummy: dummy_view(device, RASTER_FORMAT),
            dummy_acc: dummy_view(device, ACC_FORMAT),
            dummy_buffer: buffer(device, queue, wgpu::BufferUsages::STORAGE, &[0; 16]),
            draw_layout,
            filter_layout,
            mask_layout,
            apply_layout,
            apply_module,
        }
    }

    /// The copy-to-swapchain pipeline for `format`, made on first use.
    pub(crate) fn present_for(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> &wgpu::RenderPipeline {
        let apply_layout = &self.apply_layout;
        let apply_module = &self.apply_module;
        self.present.entry(format).or_insert_with(|| {
            let over = blend(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha);
            let pl = layout(device, &[apply_layout]);
            pipeline(
                device,
                &pl,
                apply_module,
                ("vs_full", "fs_present"),
                format,
                Some(over),
            )
        })
    }
}
