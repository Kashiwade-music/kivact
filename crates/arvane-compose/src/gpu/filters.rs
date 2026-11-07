//! Role: Records the layer filters of a plan (design §8.8).
//!
//! The gaussian blur, the drop shadow, the colour matrix and the filter shader.

use super::pass::{CLEAR, FrameTextures, blur_sigmas, copy, size_f, view, with_pass};
use super::pipelines::{Words, buffer};
use super::state::GpuContext;
use super::targets;
use crate::ComposeError;
use kivact_types::Rgba;
use kivact_types::geometry::{Point, Size};
use kivact_types::pixel::ColorMatrix;
use kivact_types::render::LayerFilter;

/// The `mode` values of the filter shader (`fs_filter`).
const MODE_SHRINK_X: u32 = 1;
const MODE_SHRINK_Y: u32 = 2;
const MODE_BLUR: u32 = 3;
const MODE_SHADOW_COLOR: u32 = 4;
const MODE_SHADOW_OVER: u32 = 5;
const MODE_MATRIX: u32 = 6;
const MODE_EXPAND_X: u32 = 7;
const MODE_EXPAND_Y: u32 = 8;

/// The σ (px) above which a blur axis is halved first.
const BLUR_LIMIT: f64 = 4.0;
/// The most halvings a blur axis makes.
const BLUR_LEVELS: u32 = 6;

/// The arguments of one filter pass (the uniform of `filter.wgsl`).
#[derive(Default)]
pub(super) struct Pass {
    /// Size of the output in px.
    out: (f32, f32),
    /// Size of the source in px.
    src: (f32, f32),
    /// Blur direction in px.
    dir: (f32, f32),
    /// Premultiplied colour of the shadow.
    color: [f32; 4],
    /// Shadow offset in px.
    offset: (f32, f32),
    /// Blur radius in px.
    radius: u32,
    /// The shader's `mode`.
    mode: u32,
    /// The colour matrix, row-major 4 × 5.
    matrix: [f32; 20],
}

/// The layer geometry of a filter: `px_per_unit` px per layer unit, the layer point `origin` at px (0, 0) and the layer `bounds`.
#[derive(Clone, Copy)]
pub(super) struct FilterGeometry {
    /// Px per layer unit.
    pub(super) px_per_unit: f64,
    /// The layer point at px (0, 0).
    pub(super) origin: Point,
    /// The bounds of the layer.
    pub(super) bounds: Size,
}

impl GpuContext {
    /// Record one filter of a layer from `input` into `output` (same size), in the layer `geometry`.
    pub(super) fn record_filter(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        input: u32,
        output: u32,
        filter: &LayerFilter,
        geometry: FilterGeometry,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        let FilterGeometry {
            px_per_unit,
            origin,
            bounds,
        } = geometry;
        let source = frame.target(input)?;
        let dest = frame.target(output)?;
        if (source.width(), source.height()) != (dest.width(), dest.height()) {
            return Err(ComposeError::InvalidComposition(
                "a filter's input and output differ in size".into(),
            ));
        }
        match filter {
            LayerFilter::Blur { std_dev } => {
                let sigma = std_dev * px_per_unit;
                match blur_sigmas(sigma, sigma) {
                    Some((sx, sy)) => {
                        let blurred = self.blur_axes(encoder, source, sx, sy)?;
                        copy(encoder, &blurred, dest);
                    }
                    None => copy(encoder, source, dest),
                }
            }
            LayerFilter::DropShadow {
                offset,
                std_dev,
                color,
            } => {
                let silhouette = self.shadow(encoder, source, *color)?;
                let sigma = std_dev * px_per_unit;
                let blurred = match blur_sigmas(sigma, sigma) {
                    Some((sx, sy)) => self.blur_axes(encoder, &silhouette, sx, sy)?,
                    None => silhouette,
                };
                let shift = (
                    (offset.x * px_per_unit) as f32,
                    (offset.y * px_per_unit) as f32,
                );
                self.shadow_over(encoder, dest, source, &blurred, shift)?;
            }
            LayerFilter::ColorMatrix(matrix) => {
                self.color_matrix_into(encoder, dest, source, matrix)?;
            }
            LayerFilter::Graph(graph) => {
                self.record_graph(encoder, source, dest, graph, px_per_unit, origin)?;
            }
            LayerFilter::Shader(shader) => {
                self.record_filter_shader(
                    encoder,
                    source,
                    dest,
                    shader,
                    (px_per_unit, origin, bounds),
                )?;
            }
        }
        Ok(())
    }

    /// Run one pass of the filter shader from `src` (and `aux`) into `out`.
    fn filter_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        out: &wgpu::Texture,
        src: &wgpu::Texture,
        aux: Option<&wgpu::Texture>,
        args: Pass,
    ) -> Result<(), ComposeError> {
        self.filter_pass_with(encoder, out, src, aux, args, None)
    }

    /// [`GpuContext::filter_pass`] with the blur weights in `weights`.
    fn filter_pass_with(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        out: &wgpu::Texture,
        src: &wgpu::Texture,
        aux: Option<&wgpu::Texture>,
        mut args: Pass,
        weights: Option<&wgpu::Buffer>,
    ) -> Result<(), ComposeError> {
        // The sizes come from the textures, so every mode samples the right extent.
        args.out = size_f(out);
        args.src = size_f(src);
        let mut words = Words::default();
        words
            .v4([args.out.0, args.out.1, 0.0, 0.0])
            .v4([args.src.0, args.src.1, 0.0, 0.0])
            .v4([args.dir.0, args.dir.1, 0.0, 0.0])
            .v4(args.color)
            .v4([args.offset.0, args.offset.1, 0.0, 0.0])
            .u(args.radius)
            .u(args.mode)
            .u(0)
            .u(0);
        for row in args.matrix.as_chunks::<4>().0 {
            words.v4([row[0], row[1], row[2], row[3]]);
        }
        let params = super::pass::uniform(&self.device, &self.queue, words.bytes());
        let src_view = view(src);
        let aux_view = aux.map(view);
        let aux_ref = aux_view.as_ref().unwrap_or(&self.pipelines.dummy);
        let storage = match weights {
            Some(buffer) => buffer.as_entire_binding(),
            None => self.pipelines.dummy_buffer.as_entire_binding(),
        };
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Arvane filter"),
            layout: &self.pipelines.filter_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(aux_ref),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: storage,
                },
            ],
        });
        let out_view = view(out);
        with_pass(encoder, &out_view, CLEAR, |pass| {
            pass.set_pipeline(&self.pipelines.filter);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        });
        Ok(())
    }

    /// The premultiplied colour `color` where `source` is opaque (the drop shadow's silhouette), in a new texture.
    pub(super) fn shadow(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        color: Rgba,
    ) -> Result<wgpu::Texture, ComposeError> {
        let shadow = targets::target(&self.device, (source.width(), source.height()));
        let args = Pass {
            out: size_f(&shadow),
            src: size_f(source),
            color: [
                (color.r * color.a) as f32,
                (color.g * color.a) as f32,
                (color.b * color.a) as f32,
                color.a as f32,
            ],
            mode: MODE_SHADOW_COLOR,
            ..Pass::default()
        };
        self.filter_pass(encoder, &shadow, source, None, args)?;
        Ok(shadow)
    }

    /// The drop shadow of `input` over its `shadow`, shifted by `offset` px, into `out` (design §8.8.2).
    pub(super) fn shadow_over(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        out: &wgpu::Texture,
        input: &wgpu::Texture,
        shadow: &wgpu::Texture,
        offset: (f32, f32),
    ) -> Result<(), ComposeError> {
        let args = Pass {
            mode: MODE_SHADOW_OVER,
            offset,
            ..Pass::default()
        };
        self.filter_pass(encoder, out, input, Some(shadow), args)
    }

    /// The colour matrix of `src` into `out` (design §8.8.3).
    pub(super) fn color_matrix_into(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        out: &wgpu::Texture,
        src: &wgpu::Texture,
        matrix: &ColorMatrix,
    ) -> Result<(), ComposeError> {
        let mut coefficients = [0.0_f32; 20];
        for (c, v) in coefficients.iter_mut().zip(matrix.0) {
            *c = v as f32;
        }
        let args = Pass {
            mode: MODE_MATRIX,
            matrix: coefficients,
            ..Pass::default()
        };
        self.filter_pass(encoder, out, src, None, args)
    }

    /// The gaussian blur of `src` with the standard deviations `sigma_x` and `sigma_y` px, in a new texture (design §8.8.1).
    /// Each axis is halved on its own, as the CPU does.
    pub(super) fn blur_axes(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::Texture,
        sigma_x: f64,
        sigma_y: f64,
    ) -> Result<wgpu::Texture, ComposeError> {
        let (kx, ky) = (levels(sigma_x), levels(sigma_y));
        let mut current = src.clone();
        let mut widths = Vec::with_capacity(kx as usize);
        let mut heights = Vec::with_capacity(ky as usize);
        for _ in 0..kx {
            widths.push(current.width());
            current = self.shrink(encoder, &current, MODE_SHRINK_X)?;
        }
        for _ in 0..ky {
            heights.push(current.height());
            current = self.shrink(encoder, &current, MODE_SHRINK_Y)?;
        }
        if sigma_x >= 0.5 {
            let sigma = sigma_x / f64::from(1_u32 << kx);
            current = self.convolve_axis(encoder, &current, sigma, (1.0, 0.0))?;
        }
        if sigma_y >= 0.5 {
            let sigma = sigma_y / f64::from(1_u32 << ky);
            current = self.convolve_axis(encoder, &current, sigma, (0.0, 1.0))?;
        }
        for &w in widths.iter().rev() {
            let h = current.height();
            current = self.resample(encoder, &current, (w, h), MODE_EXPAND_X)?;
        }
        for &h in heights.iter().rev() {
            let w = current.width();
            current = self.resample(encoder, &current, (w, h), MODE_EXPAND_Y)?;
        }
        Ok(current)
    }

    /// `src` halved along one axis (`mode` MODE_SHRINK_X or MODE_SHRINK_Y), in a new texture.
    fn shrink(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::Texture,
        mode: u32,
    ) -> Result<wgpu::Texture, ComposeError> {
        let (w, h) = (src.width(), src.height());
        let size = if mode == MODE_SHRINK_X {
            (w.div_ceil(2), h)
        } else {
            (w, h.div_ceil(2))
        };
        self.resample(encoder, src, size, mode)
    }

    /// `src` resampled by `mode` into a new texture of `size` px.
    fn resample(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::Texture,
        size: (u32, u32),
        mode: u32,
    ) -> Result<wgpu::Texture, ComposeError> {
        let out = targets::target(&self.device, size);
        let args = Pass {
            mode,
            ..Pass::default()
        };
        self.filter_pass(encoder, &out, src, None, args)?;
        Ok(out)
    }

    /// `src` convolved along `dir` with the gaussian of `sigma` px, in a new texture.
    fn convolve_axis(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::Texture,
        sigma: f64,
        dir: (f32, f32),
    ) -> Result<wgpu::Texture, ComposeError> {
        let (radius, weights) = gaussian(sigma);
        let weight_buffer = buffer(
            &self.device,
            &self.queue,
            wgpu::BufferUsages::STORAGE,
            &weights,
        );
        let out = targets::target(&self.device, (src.width(), src.height()));
        let args = Pass {
            dir,
            radius,
            mode: MODE_BLUR,
            ..Pass::default()
        };
        self.filter_pass_with(encoder, &out, src, None, args, Some(&weight_buffer))?;
        Ok(out)
    }
}

/// The halvings of an axis with σ (px): while σ / 2^k > 4, at most six.
fn levels(sigma: f64) -> u32 {
    let mut k = 0;
    while k < BLUR_LEVELS && sigma / f64::from(1_u32 << k) > BLUR_LIMIT {
        k += 1;
    }
    k
}

/// The radius and the normalised weights of a gaussian of `sigma` px: w_i = exp(-i² / 2σ²) over [-r, r], r = ⌈3σ⌉.
fn gaussian(sigma: f64) -> (u32, Vec<u8>) {
    let radius = (3.0 * sigma).ceil().max(1.0) as u32;
    let r = i64::from(radius);
    let raw: Vec<f64> = (-r..=r)
        .map(|i| (-((i * i) as f64) / (2.0 * sigma * sigma)).exp())
        .collect();
    let total: f64 = raw.iter().sum();
    let mut bytes = Vec::with_capacity(raw.len() * 4);
    for w in raw {
        bytes.extend_from_slice(&((w / total) as f32).to_le_bytes());
    }
    (radius, bytes)
}
