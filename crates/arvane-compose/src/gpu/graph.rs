//! Role: Records a filter graph's primitives (design §8.8.4).
//!
//! One pass of the graph shader per primitive, over textures of the region's size.

use super::pass::{CLEAR, blend_index, blur_sigmas, copy, uniform, view, with_pass};
use super::pipelines::{Words, buffer};
use super::state::GpuContext;
use super::targets;
use crate::ComposeError;
use crate::cpu::graph::turbulence::{self, B_LEN};
use crate::cpu::graph::{LINEAR_RGB_TO_SRGB_TABLE, SRGB_TO_LINEAR_RGB_TABLE};
use arvane_render::RASTER_FORMAT;
use kivact_types::Rgba;
use kivact_types::effect::{
    Channel, CompositeOp, EdgeMode, FilterColorSpace, FilterGraph, FilterInput, Light,
    MorphologyOp, PrimitiveOp, TransferFn, TurbulenceKind,
};
use kivact_types::geometry::{Point, Rect};
use kivact_types::pixel::{BlendMode, ColorMatrix, Pixels};

/// The parameters of a convolution (feConvolveMatrix).
struct Convolve<'a> {
    order: &'a (u32, u32),
    kernel: &'a [f64],
    divisor: &'a f64,
    bias: &'a f64,
    target: &'a (u32, u32),
    edge: &'a EdgeMode,
    preserve_alpha: &'a bool,
}

/// The modes of the graph shader (`graph_ops.wgsl`).
const M_SOURCE_ALPHA: u32 = 1;
const M_CONVERT: u32 = 2;
const M_BLEND: u32 = 3;
const M_COMPOSITE: u32 = 4;
const M_ARITHMETIC: u32 = 5;
const M_FLOOD: u32 = 6;
const M_IMAGE: u32 = 7;
const M_OFFSET: u32 = 8;
const M_TILE: u32 = 9;
const M_TRANSFER: u32 = 10;
const M_DEMULTIPLY: u32 = 11;
const M_CONVOLVE: u32 = 12;
const M_LIGHT: u32 = 13;
const M_DISPLACE: u32 = 14;
const M_MORPH: u32 = 15;
const M_TURBULENCE: u32 = 16;
const M_CLEAR: u32 = 17;

/// Offsets of the tables in the tables buffer (`graph_common.wgsl`).
const LUT_TO_LINEAR: usize = 0;
const LUT_TO_SRGB: usize = 256;
const KERNEL: usize = 512;
const TRANSFER_INFO: usize = 768;
const TRANSFER_TABLE: usize = 800;
const TABLES_LEN: usize = TRANSFER_TABLE + 4 * 256;

/// The longest kernel and transfer table (the shader's limit).
const TABLE_MAX: usize = 256;

/// Kind values of the light sources (`graph_light.wgsl`).
const LIGHT_DISTANT: f32 = 0.0;
const LIGHT_POINT: f32 = 1.0;
const LIGHT_SPOT: f32 = 2.0;

/// The cone value that means no limiting cone (below any cosine).
const NO_CONE: f32 = -2.0;

/// A texture of the graph with its colour space.
#[derive(Clone)]
struct Img {
    /// The premultiplied pixels of the region.
    tex: wgpu::Texture,
    /// The colour space they are in.
    cs: FilterColorSpace,
}

/// The parameters of one pass: its mode, the real parameters `p` and the integers (`ints`, after the mode).
#[derive(Clone, Copy, Default)]
struct Op {
    /// The mode (`M_*`).
    mode: u32,
    /// The real parameters p0..p3.
    p: [[f32; 4]; 4],
    /// `ints.y`, `ints.z`, `ints.w`, `ints2.x`, `ints2.y`, `ints2.z`.
    ints: [u32; 6],
}

impl Op {
    /// An operation of `mode` with no parameters.
    fn new(mode: u32) -> Self {
        Op {
            mode,
            ..Op::default()
        }
    }

    /// The uniform bytes of the pass (`GraphParams`, 144 bytes) for an output of `out` px and inputs of `a` and `b` px.
    fn bytes(&self, out: (u32, u32), a: (u32, u32), b: (u32, u32)) -> Vec<u8> {
        let mut words = Words::default();
        words
            .v4([out.0 as f32, out.1 as f32, 0.0, 0.0])
            .v4([a.0 as f32, a.1 as f32, 0.0, 0.0])
            .v4([b.0 as f32, b.1 as f32, 0.0, 0.0]);
        for p in self.p {
            words.v4(p);
        }
        words
            .u(self.mode)
            .u(self.ints[0])
            .u(self.ints[1])
            .u(self.ints[2])
            .u(self.ints[3])
            .u(self.ints[4])
            .u(self.ints[5])
            .u(0);
        words.bytes().to_vec()
    }
}

/// The storage data of a pass: the tables, and the turbulence lattice and gradients.
struct Data {
    /// The floats of `graph_common.wgsl`'s tables.
    tables: Vec<f32>,
    /// The turbulence lattice selector.
    lattice: Vec<u32>,
    /// The turbulence gradients, channel by channel.
    gradient: Vec<[f32; 2]>,
}

impl Data {
    /// The data with the colour conversion tables and nothing else.
    fn new() -> Self {
        let mut tables = vec![0.0; TABLES_LEN];
        for (i, (to_linear, to_srgb)) in SRGB_TO_LINEAR_RGB_TABLE
            .iter()
            .zip(LINEAR_RGB_TO_SRGB_TABLE.iter())
            .enumerate()
        {
            tables[LUT_TO_LINEAR + i] = f32::from(*to_linear);
            tables[LUT_TO_SRGB + i] = f32::from(*to_srgb);
        }
        Data {
            tables,
            lattice: vec![0; B_LEN],
            gradient: vec![[0.0; 2]; 4 * B_LEN],
        }
    }

    /// The convolution kernel of the tables (its values beyond the shader's limit are dropped).
    fn with_kernel(mut self, kernel: &[f64]) -> Self {
        for (slot, value) in self.tables[KERNEL..KERNEL + TABLE_MAX]
            .iter_mut()
            .zip(kernel)
        {
            *slot = *value as f32;
        }
        self
    }

    /// The transfer function of channel `ch` (0 r, 1 g, 2 b, 3 a) in the tables.
    fn set_transfer(&mut self, ch: usize, func: &TransferFn) {
        let (kind, params, table): (f32, [f32; 3], &[f64]) = match func {
            TransferFn::Identity => (0.0, [0.0; 3], &[]),
            TransferFn::Table(values) => (1.0, [0.0; 3], values),
            TransferFn::Discrete(values) => (2.0, [0.0; 3], values),
            TransferFn::Linear { slope, intercept } => {
                (3.0, [*slope as f32, *intercept as f32, 0.0], &[])
            }
            TransferFn::Gamma {
                amplitude,
                exponent,
                offset,
            } => (
                4.0,
                [*amplitude as f32, *exponent as f32, *offset as f32],
                &[],
            ),
        };
        // An empty table is the identity, as the CPU reads it.
        let kind = if table.is_empty() && (kind == 1.0 || kind == 2.0) {
            0.0
        } else {
            kind
        };
        let base = TRANSFER_INFO + ch * 8;
        let table_at = TRANSFER_TABLE + ch * TABLE_MAX;
        self.tables[base] = kind;
        self.tables[base + 1..base + 4].copy_from_slice(&params);
        self.tables[base + 4] = table.len() as f32;
        self.tables[base + 5] = table_at as f32;
        for (slot, value) in self.tables[table_at..table_at + TABLE_MAX]
            .iter_mut()
            .zip(table)
        {
            *slot = *value as f32;
        }
    }

    /// The turbulence lattice and gradients of `seed` (the CPU builds them, design §8.8.4).
    fn set_turbulence(&mut self, seed: i32) {
        let built = turbulence::init(seed);
        self.lattice = built.lattice.iter().map(|&i| i as u32).collect();
        for (ch, gradient) in built.gradient.iter().enumerate() {
            for (i, g) in gradient.iter().enumerate() {
                self.gradient[ch * B_LEN + i] = [g[0] as f32, g[1] as f32];
            }
        }
    }

    /// The storage buffers of the data: tables, lattice and gradients.
    fn buffers(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> [wgpu::Buffer; 3] {
        let storage = wgpu::BufferUsages::STORAGE;
        let tables: Vec<u8> = self.tables.iter().flat_map(|v| v.to_le_bytes()).collect();
        let lattice: Vec<u8> = self.lattice.iter().flat_map(|v| v.to_le_bytes()).collect();
        let gradient: Vec<u8> = self
            .gradient
            .iter()
            .flat_map(|g| g[0].to_le_bytes().into_iter().chain(g[1].to_le_bytes()))
            .collect();
        [
            buffer(device, queue, storage, &tables),
            buffer(device, queue, storage, &lattice),
            buffer(device, queue, storage, &gradient),
        ]
    }
}

/// One run of a graph over a region: the source, the results so far and the mapping to px.
struct Run<'a> {
    /// The executor, for its device and pipelines.
    gpu: &'a GpuContext,
    /// The frame's encoder; every pass is recorded into it.
    encoder: &'a mut wgpu::CommandEncoder,
    /// The layer's pixels, premultiplied sRGB.
    source: wgpu::Texture,
    /// The region's size in px.
    size: (u32, u32),
    /// Device px per layer unit.
    ppu: f64,
    /// The layer point at px (0, 0).
    origin: Point,
    /// The results of the primitives run so far.
    results: Vec<Img>,
    /// The buffers of the default data (the colour conversion tables).
    tables: [wgpu::Buffer; 3],
}

impl Run<'_> {
    /// Record one pass of `op` into a new texture of the region: `a` and `b` are its inputs (`extra`: the data buffers, if it reads any).
    fn pass(
        &mut self,
        op: &Op,
        extra: Option<&[wgpu::Buffer; 3]>,
        a: Option<&wgpu::Texture>,
        b: Option<&wgpu::Texture>,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let out = targets::target(&self.gpu.device, self.size);
        let buffers = extra.unwrap_or(&self.tables);
        self.gpu.draw_graph(self.encoder, op, buffers, &out, a, b)?;
        Ok(Img { tex: out, cs })
    }

    /// The result of `input` in its own colour space.
    fn fetch(&mut self, input: FilterInput) -> Result<Img, ComposeError> {
        match input {
            FilterInput::Source => Ok(Img {
                tex: self.source.clone(),
                cs: FilterColorSpace::Srgb,
            }),
            FilterInput::SourceAlpha => {
                let source = self.source.clone();
                self.pass(
                    &Op::new(M_SOURCE_ALPHA),
                    None,
                    Some(&source),
                    None,
                    FilterColorSpace::Srgb,
                )
            }
            FilterInput::Result(i) => self.results.get(i as usize).cloned().ok_or_else(|| {
                ComposeError::InvalidComposition(format!("filter result {i} is not computed yet"))
            }),
        }
    }

    /// The result of `input` in colour space `cs`.
    fn fetch_in(&mut self, input: FilterInput, cs: FilterColorSpace) -> Result<Img, ComposeError> {
        let img = self.fetch(input)?;
        self.space(img, cs)
    }

    /// `img` in colour space `cs`, converted when it differs (the CPU's `into_space`).
    fn space(&mut self, img: Img, cs: FilterColorSpace) -> Result<Img, ComposeError> {
        if img.cs == cs {
            return Ok(img);
        }
        let mut op = Op::new(M_CONVERT);
        op.ints[0] = match cs {
            FilterColorSpace::Srgb => LUT_TO_SRGB as u32,
            FilterColorSpace::LinearRgb => LUT_TO_LINEAR as u32,
        };
        self.pass(&op, None, Some(&img.tex), None, cs)
    }

    /// The layer point `(x, y)` in px.
    fn layer_to_px(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.origin.x) * self.ppu,
            (y - self.origin.y) * self.ppu,
        )
    }

    /// The layer point `p` (with its z) in px and z.
    fn point_px(&self, p: [f64; 3]) -> [f32; 4] {
        let (x, y) = self.layer_to_px(p[0], p[1]);
        [x as f32, y as f32, (p[2] * self.ppu) as f32, 0.0]
    }

    /// One filter primitive computed in colour space `cs` (the CPU's `run`).
    fn primitive(
        &mut self,
        op: &PrimitiveOp<Pixels>,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        match op {
            PrimitiveOp::Blend { top, bottom, mode } => self.blend(*top, *bottom, *mode, cs),
            PrimitiveOp::ColorMatrix { input, matrix } => self.color_matrix(*input, matrix, cs),
            PrimitiveOp::ComponentTransfer { input, r, g, b, a } => {
                self.component_transfer(*input, [r, g, b, a], cs)
            }
            PrimitiveOp::Composite { top, bottom, op } => self.composite(*top, *bottom, *op, cs),
            PrimitiveOp::Flood { color } => {
                let mut pass = Op::new(M_FLOOD);
                pass.p[0] = premultiplied(*color);
                self.pass(&pass, None, None, None, FilterColorSpace::Srgb)
            }
            PrimitiveOp::Image { image, rect } => self.image(image, *rect),
            PrimitiveOp::Merge { inputs } => self.merge(inputs, cs),
            PrimitiveOp::Offset { input, offset } => self.offset(*input, offset),
            PrimitiveOp::Tile { input, tile } => self.tile(*input, tile, cs),
            other => self.filter_primitive(other, cs),
        }
    }

    /// The filter-kernel, lighting and turbulence primitives (the rest of `primitive`).
    fn filter_primitive(
        &mut self,
        op: &PrimitiveOp<Pixels>,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        match op {
            PrimitiveOp::ConvolveMatrix {
                input,
                order,
                kernel,
                divisor,
                bias,
                target,
                edge,
                preserve_alpha,
            } => {
                let conv = Convolve {
                    order,
                    kernel,
                    divisor,
                    bias,
                    target,
                    edge,
                    preserve_alpha,
                };
                self.convolve(*input, conv, cs)
            }
            PrimitiveOp::DiffuseLighting {
                input,
                surface_scale,
                diffuse_constant,
                color,
                light,
            } => {
                let lighting = Lighting::diffuse(*surface_scale, *diffuse_constant, *color, light);
                self.light(*input, lighting, cs)
            }
            PrimitiveOp::SpecularLighting {
                input,
                surface_scale,
                specular_constant,
                specular_exponent,
                color,
                light,
            } => {
                let lighting = Lighting::specular(
                    *surface_scale,
                    *specular_constant,
                    *specular_exponent,
                    *color,
                    light,
                );
                self.light(*input, lighting, cs)
            }
            PrimitiveOp::DisplacementMap {
                input,
                map,
                scale,
                x,
                y,
            } => self.displace(*input, *map, *scale, (*x, *y), cs),
            PrimitiveOp::DropShadow {
                input,
                offset,
                std_dev,
                color,
            } => self.drop_shadow(*input, offset, *std_dev, *color, cs),
            PrimitiveOp::GaussianBlur { input, std_dev, .. } => self.gaussian(*input, *std_dev, cs),
            PrimitiveOp::Morphology { input, op, radius } => {
                self.morphology(*input, *op, *radius, cs)
            }
            PrimitiveOp::Turbulence {
                base_frequency,
                octaves,
                seed,
                kind,
                stitch,
            } => self.turbulence(*base_frequency, *octaves, *seed, *kind, *stitch, cs),
            other => self.primitive(other, cs),
        }
    }

    /// feBlend of `top` over `bottom`.
    fn blend(
        &mut self,
        top: FilterInput,
        bottom: FilterInput,
        mode: BlendMode,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let dst = self.fetch_in(bottom, cs)?;
        let src = self.fetch_in(top, cs)?;
        let mut pass = Op::new(M_BLEND);
        pass.ints[0] = blend_index(mode);
        self.pass(&pass, None, Some(&src.tex), Some(&dst.tex), cs)
    }

    /// feColorMatrix of `input`.
    fn color_matrix(
        &mut self,
        input: FilterInput,
        matrix: &ColorMatrix,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let img = self.fetch_in(input, cs)?;
        let out = targets::target(&self.gpu.device, self.size);
        self.gpu
            .color_matrix_into(self.encoder, &out, &img.tex, matrix)?;
        Ok(Img { tex: out, cs })
    }

    /// feComponentTransfer of `input` with the functions of the channels r, g, b and a.
    fn component_transfer(
        &mut self,
        input: FilterInput,
        [r, g, b, a]: [&TransferFn; 4],
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let img = self.fetch_in(input, cs)?;
        let mut data = Data::new();
        data.set_transfer(0, r);
        data.set_transfer(1, g);
        data.set_transfer(2, b);
        data.set_transfer(3, a);
        let buffers = data.buffers(&self.gpu.device, &self.gpu.queue);
        self.pass(
            &Op::new(M_TRANSFER),
            Some(&buffers),
            Some(&img.tex),
            None,
            cs,
        )
    }

    /// feComposite of `top` and `bottom` by `composite`.
    fn composite(
        &mut self,
        top: FilterInput,
        bottom: FilterInput,
        composite: CompositeOp,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let first = self.fetch_in(top, cs)?;
        let second = self.fetch_in(bottom, cs)?;
        let mut pass = Op::new(M_COMPOSITE);
        if let CompositeOp::Arithmetic { k1, k2, k3, k4 } = composite {
            pass = Op::new(M_ARITHMETIC);
            pass.p[0] = [k1 as f32, k2 as f32, k3 as f32, k4 as f32];
        } else {
            pass.ints[0] = composite_index(composite);
        }
        self.pass(&pass, None, Some(&first.tex), Some(&second.tex), cs)
    }

    /// feMerge: the inputs composited over each other in order.
    fn merge(&mut self, inputs: &[FilterInput], cs: FilterColorSpace) -> Result<Img, ComposeError> {
        let mut acc = self.pass(&Op::new(M_CLEAR), None, None, None, cs)?;
        for &input in inputs {
            let src = self.fetch_in(input, cs)?;
            let mut pass = Op::new(M_COMPOSITE);
            pass.ints[0] = composite_index(CompositeOp::Over);
            acc = self.pass(&pass, None, Some(&src.tex), Some(&acc.tex), cs)?;
        }
        Ok(acc)
    }

    /// feOffset of `input` by `offset` (layer units).
    fn offset(&mut self, input: FilterInput, offset: &Point) -> Result<Img, ComposeError> {
        let img = self.fetch(input)?;
        let dx = (offset.x * self.ppu) as i64;
        let dy = (offset.y * self.ppu) as i64;
        if dx == 0 && dy == 0 {
            return Ok(img);
        }
        let mut pass = Op::new(M_OFFSET);
        pass.p[0] = [dx as f32, dy as f32, 0.0, 0.0];
        self.pass(&pass, None, Some(&img.tex), None, img.cs)
    }

    /// feTile of `input` over the tile rectangle `tile`.
    fn tile(
        &mut self,
        input: FilterInput,
        tile: &Rect,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let img = self.fetch(input)?;
        let (x, y) = self.layer_to_px(tile.x, tile.y);
        let mut pass = Op::new(M_TILE);
        pass.p[0] = [
            x as f32,
            y as f32,
            (tile.width * self.ppu) as f32,
            (tile.height * self.ppu) as f32,
        ];
        self.pass(&pass, None, Some(&img.tex), None, cs)
    }

    /// feConvolveMatrix of `input`.
    fn convolve(
        &mut self,
        input: FilterInput,
        conv: Convolve<'_>,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let mut img = self.fetch_in(input, cs)?;
        if *conv.preserve_alpha {
            img = self.pass(&Op::new(M_DEMULTIPLY), None, Some(&img.tex), None, cs)?;
        }
        let divisor = if *conv.divisor == 0.0 {
            let sum: f64 = conv.kernel.iter().sum();
            if sum == 0.0 { 1.0 } else { sum }
        } else {
            *conv.divisor
        };
        let buffers = Data::new()
            .with_kernel(conv.kernel)
            .buffers(&self.gpu.device, &self.gpu.queue);
        let mut pass = Op::new(M_CONVOLVE);
        pass.p[0] = [divisor as f32, *conv.bias as f32, 0.0, 0.0];
        pass.ints = [
            conv.order.0,
            conv.order.1,
            conv.target.0,
            conv.target.1,
            edge_index(*conv.edge),
            u32::from(*conv.preserve_alpha),
        ];
        self.pass(&pass, Some(&buffers), Some(&img.tex), None, cs)
    }

    /// The lighting primitives: `lighting` of the alpha surface of `input`.
    fn light(
        &mut self,
        input: FilterInput,
        lighting: Lighting<'_>,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let img = self.fetch(input)?;
        self.lighting(&img, lighting, cs)
    }

    /// feDisplacementMap of `input` by `map`, scaled by `scale` (layer units) on the channels `axes`.
    fn displace(
        &mut self,
        input: FilterInput,
        map: FilterInput,
        scale: f64,
        axes: (Channel, Channel),
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let src = self.fetch_in(input, cs)?;
        let map = self.fetch_in(map, cs)?;
        let s = (scale * self.ppu) as f32;
        let mut pass = Op::new(M_DISPLACE);
        pass.p[0] = [s, s, 0.0, 0.0];
        pass.ints[0] = channel_index(axes.0);
        pass.ints[1] = channel_index(axes.1);
        self.pass(&pass, None, Some(&src.tex), Some(&map.tex), cs)
    }

    /// feDropShadow of `input`: its silhouette in `color`, blurred and moved by `offset`.
    fn drop_shadow(
        &mut self,
        input: FilterInput,
        offset: &Point,
        std_dev: (f64, f64),
        color: Rgba,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let img = self.fetch_in(input, cs)?;
        let tint = color_in(color, cs);
        let silhouette = self.gpu.shadow(self.encoder, &img.tex, tint)?;
        let (sx, sy) = (std_dev.0 * self.ppu, std_dev.1 * self.ppu);
        let blurred = match blur_sigmas(sx, sy) {
            Some((sx, sy)) => self.gpu.blur_axes(self.encoder, &silhouette, sx, sy)?,
            None => silhouette,
        };
        let shift = ((offset.x * self.ppu) as f32, (offset.y * self.ppu) as f32);
        let out = targets::target(&self.gpu.device, self.size);
        self.gpu
            .shadow_over(self.encoder, &out, &img.tex, &blurred, shift)?;
        Ok(Img { tex: out, cs })
    }

    /// feGaussianBlur of `input` with the deviations `std_dev` (layer units).
    fn gaussian(
        &mut self,
        input: FilterInput,
        std_dev: (f64, f64),
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let img = self.fetch_in(input, cs)?;
        match blur_sigmas(std_dev.0 * self.ppu, std_dev.1 * self.ppu) {
            Some((sx, sy)) => {
                let tex = self.gpu.blur_axes(self.encoder, &img.tex, sx, sy)?;
                Ok(Img { tex, cs })
            }
            None => Ok(img),
        }
    }

    /// feMorphology of `input` by `op` over `radius` (layer units).
    fn morphology(
        &mut self,
        input: FilterInput,
        op: MorphologyOp,
        radius: (f64, f64),
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let img = self.fetch_in(input, cs)?;
        let (rx, ry) = (radius.0 * self.ppu, radius.1 * self.ppu);
        let pass = if rx > 0.0 && ry > 0.0 {
            let mut pass = Op::new(M_MORPH);
            pass.p[0] = [rx as f32, ry as f32, 0.0, 0.0];
            pass.ints[0] = match op {
                MorphologyOp::Erode => 0,
                MorphologyOp::Dilate => 1,
            };
            pass
        } else {
            Op::new(M_CLEAR)
        };
        self.pass(&pass, None, Some(&img.tex), None, cs)
    }

    /// feTurbulence of the given frequency, octaves, seed, kind and stitching.
    fn turbulence(
        &mut self,
        base_frequency: (f64, f64),
        octaves: u32,
        seed: f64,
        kind: TurbulenceKind,
        stitch: bool,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let mut data = Data::new();
        data.set_turbulence(seed.trunc() as i32);
        let buffers = data.buffers(&self.gpu.device, &self.gpu.queue);
        let mut pass = Op::new(M_TURBULENCE);
        pass.p[0] = [
            (self.origin.x * self.ppu) as f32,
            (self.origin.y * self.ppu) as f32,
            (base_frequency.0 / self.ppu) as f32,
            (base_frequency.1 / self.ppu) as f32,
        ];
        pass.ints = [
            octaves,
            u32::from(stitch),
            u32::from(kind == TurbulenceKind::Turbulence),
            0,
            0,
            0,
        ];
        self.pass(&pass, Some(&buffers), None, None, cs)
    }

    /// feImage: the image stretched over its rectangle, in sRGB.
    fn image(&mut self, image: &Pixels, rect: Rect) -> Result<Img, ComposeError> {
        if image.width == 0 || image.height == 0 {
            return self.pass(&Op::new(M_CLEAR), None, None, None, FilterColorSpace::Srgb);
        }
        let texture = self.gpu.upload_premultiplied(image);
        let (x, y) = self.layer_to_px(rect.x, rect.y);
        let mut pass = Op::new(M_IMAGE);
        pass.p[0] = [
            x as f32,
            y as f32,
            (rect.width * self.ppu) as f32,
            (rect.height * self.ppu) as f32,
        ];
        pass.p[1] = [image.width as f32, image.height as f32, 0.0, 0.0];
        self.pass(&pass, None, None, Some(&texture), FilterColorSpace::Srgb)
    }

    /// feDiffuseLighting or feSpecularLighting of `img` (the alpha surface) in colour space `cs`.
    fn lighting(
        &mut self,
        img: &Img,
        lighting: Lighting<'_>,
        cs: FilterColorSpace,
    ) -> Result<Img, ComposeError> {
        let mut pass = Op::new(M_LIGHT);
        pass.ints[0] = u32::from(lighting.specular);
        let (kind, vector, cone) = match lighting.light {
            Light::Distant {
                azimuth_deg,
                elevation_deg,
            } => {
                let az = (*azimuth_deg as f32).to_radians();
                let el = (*elevation_deg as f32).to_radians();
                let v = [az.cos() * el.cos(), az.sin() * el.cos(), el.sin(), 0.0];
                (LIGHT_DISTANT, v, NO_CONE)
            }
            Light::Point { position } => (LIGHT_POINT, self.point_px(*position), NO_CONE),
            Light::Spot {
                position,
                points_at,
                exponent,
                cone_deg,
            } => {
                let at = self.point_px(*points_at);
                pass.p[2] = [at[0], at[1], at[2], *exponent as f32];
                let cone = (*cone_deg).map_or(NO_CONE, |c| (c as f32).to_radians().cos());
                (LIGHT_SPOT, self.point_px(*position), cone)
            }
        };
        pass.p[0] = [
            kind,
            lighting.surface_scale as f32,
            lighting.constant as f32,
            lighting.exponent as f32,
        ];
        pass.p[1] = vector;
        let color = lighting.color;
        pass.p[3] = [
            (color.r * 255.0) as f32,
            (color.g * 255.0) as f32,
            (color.b * 255.0) as f32,
            cone,
        ];
        self.pass(&pass, None, Some(&img.tex), None, cs)
    }
}

/// The parameters of one lighting primitive.
struct Lighting<'a> {
    /// Specular rather than diffuse.
    specular: bool,
    /// The surface scale.
    surface_scale: f64,
    /// The diffuse or specular constant.
    constant: f64,
    /// The specular exponent.
    exponent: f64,
    /// The lighting colour.
    color: Rgba,
    /// The light source.
    light: &'a Light,
}

impl<'a> Lighting<'a> {
    /// The parameters of feDiffuseLighting.
    fn diffuse(surface_scale: f64, constant: f64, color: Rgba, light: &'a Light) -> Self {
        Self {
            specular: false,
            surface_scale,
            constant,
            exponent: 0.0,
            color,
            light,
        }
    }

    /// The parameters of feSpecularLighting.
    fn specular(
        surface_scale: f64,
        constant: f64,
        exponent: f64,
        color: Rgba,
        light: &'a Light,
    ) -> Self {
        Self {
            specular: true,
            surface_scale,
            constant,
            exponent,
            color,
            light,
        }
    }
}

impl GpuContext {
    /// Record `graph` over `source` into `dest` (both the region's size) through `encoder` (design §8.8.4).
    pub(super) fn record_graph(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        dest: &wgpu::Texture,
        graph: &FilterGraph<Pixels>,
        ppu: f64,
        origin: Point,
    ) -> Result<(), ComposeError> {
        graph.validate().map_err(ComposeError::InvalidComposition)?;
        let tables = Data::new().buffers(&self.device, &self.queue);
        let mut run = Run {
            gpu: self,
            encoder,
            source: source.clone(),
            size: (source.width(), source.height()),
            ppu,
            origin,
            results: Vec::with_capacity(graph.primitives.len()),
            tables,
        };
        for primitive in &graph.primitives {
            let cs = primitive.color_space.unwrap_or(graph.color_space);
            let image = run.primitive(&primitive.op, cs)?;
            run.results.push(image);
        }
        let last = run.results.pop().ok_or_else(|| {
            ComposeError::InvalidComposition("filter graph has no primitives".into())
        })?;
        let last = run.space(last, FilterColorSpace::Srgb)?;
        copy(run.encoder, &last.tex, dest);
        Ok(())
    }

    /// Record one pass of the graph shader: `op` over `out`, reading `a` and `b` and the data `buffers`.
    fn draw_graph(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        op: &Op,
        buffers: &[wgpu::Buffer; 3],
        out: &wgpu::Texture,
        a: Option<&wgpu::Texture>,
        b: Option<&wgpu::Texture>,
    ) -> Result<(), ComposeError> {
        let size = |t: &wgpu::Texture| (t.width(), t.height());
        let bytes = op.bytes(size(out), a.map_or((0, 0), size), b.map_or((0, 0), size));
        let params = uniform(&self.device, &self.queue, &bytes);
        let a_view = a.map(view);
        let b_view = b.map(view);
        let a_ref = a_view.as_ref().unwrap_or(&self.pipelines.dummy);
        let b_ref = b_view.as_ref().unwrap_or(&self.pipelines.dummy);
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arvane filter graph"),
            layout: &self.pipelines.graph_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(a_ref),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(b_ref),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers[0].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers[1].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: buffers[2].as_entire_binding(),
                },
            ],
        });
        let out_view = view(out);
        with_pass(encoder, &out_view, CLEAR, |pass| {
            pass.set_pipeline(&self.pipelines.graph);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        });
        Ok(())
    }

    /// A texture of `image`'s size holding its pixels premultiplied (the shader reads premultiplied texels).
    fn upload_premultiplied(&self, image: &Pixels) -> wgpu::Texture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Arvane filter image"),
            size: wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: RASTER_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut bytes = Vec::with_capacity(image.rgba.len());
        for px in image.rgba.as_chunks::<4>().0 {
            let a = u32::from(px[3]);
            for c in &px[..3] {
                bytes.push((f32::from(*c) * a as f32 / 255.0).round() as u8);
            }
            bytes.push(px[3]);
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width),
                rows_per_image: Some(image.height),
            },
            wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
        );
        texture
    }
}

/// The composite operator's index in `graph_ops.wgsl` (over, in, out, atop, xor, lighter).
fn composite_index(op: CompositeOp) -> u32 {
    match op {
        CompositeOp::Over | CompositeOp::Arithmetic { .. } => 0,
        CompositeOp::In => 1,
        CompositeOp::Out => 2,
        CompositeOp::Atop => 3,
        CompositeOp::Xor => 4,
        CompositeOp::Lighter => 5,
    }
}

/// The index of an edge mode in the convolution (none, duplicate, wrap).
fn edge_index(edge: EdgeMode) -> u32 {
    match edge {
        EdgeMode::None => 0,
        EdgeMode::Duplicate => 1,
        EdgeMode::Wrap => 2,
    }
}

/// The index of a channel in a displacement map (r, g, b, a).
fn channel_index(channel: Channel) -> u32 {
    match channel {
        Channel::R => 0,
        Channel::G => 1,
        Channel::B => 2,
        Channel::A => 3,
    }
}

/// A colour as premultiplied floats.
fn premultiplied(c: Rgba) -> [f32; 4] {
    let a = c.a as f32;
    [c.r as f32 * a, c.g as f32 * a, c.b as f32 * a, a]
}

/// A shadow colour in colour space `cs` (sRGB unchanged; linear through the table), as the CPU reads it.
fn color_in(c: Rgba, cs: FilterColorSpace) -> Rgba {
    if cs == FilterColorSpace::Srgb {
        return c;
    }
    let lin = |v: f64| f64::from(SRGB_TO_LINEAR_RGB_TABLE[(v * 255.0).round() as usize]) / 255.0;
    Rgba {
        r: lin(c.r),
        g: lin(c.g),
        b: lin(c.b),
        a: c.a,
    }
}
