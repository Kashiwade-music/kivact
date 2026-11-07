//! Role: Records the filter operations of a plan: the gaussian blur, the drop shadow, the colour matrix,
//! and the pass-through of graphs and shaders (design §8.8).

use super::exec::{CLEAR, FrameTextures, size_f, view, with_pass};
use super::pipelines::{Words, buffer};
use super::state::GpuContext;
use super::targets;
use crate::ComposeError;
use kivact_types::render::LayerFilter;

/// The `mode` values of the filter shader (`fs_filter`).
const MODE_DOWNSCALE: u32 = 1;
const MODE_UPSCALE: u32 = 2;
const MODE_BLUR: u32 = 3;
const MODE_SHADOW_COLOR: u32 = 4;
const MODE_SHADOW_OVER: u32 = 5;
const MODE_MATRIX: u32 = 6;

/// The largest σ (px) the blur filters at full size; above it the image is halved first.
const BLUR_LIMIT: f64 = 4.0;
/// The most halvings the blur makes.
const BLUR_LEVELS: u32 = 6;

/// The arguments of one filter pass (the uniform of `filter.wgsl`).
#[derive(Default)]
struct Pass {
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

impl GpuContext {
    /// Record one filter from `input` into `output` (same size).
    pub(super) fn record_filter(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        input: u32,
        output: u32,
        filter: &LayerFilter,
        px_per_unit: f64,
        frame: &FrameTextures<'_>,
    ) -> Result<(), ComposeError> {
        let source = frame.target(input)?;
        let dest = frame.target(output)?;
        if (source.width(), source.height()) != (dest.width(), dest.height()) {
            return Err(ComposeError::InvalidComposition(
                "a filter's input and output differ in size".into(),
            ));
        }
        match filter {
            LayerFilter::Blur { std_dev } => {
                let blurred = self.blur(encoder, source, std_dev * px_per_unit)?;
                copy(encoder, &blurred, dest);
            }
            LayerFilter::DropShadow {
                offset,
                std_dev,
                color,
            } => {
                let shadow = self.shadow(encoder, source, *color)?;
                let blurred = self.blur(encoder, &shadow, std_dev * px_per_unit)?;
                let args = Pass {
                    mode: MODE_SHADOW_OVER,
                    offset: (
                        (offset.x * px_per_unit) as f32,
                        (offset.y * px_per_unit) as f32,
                    ),
                    ..Pass::default()
                };
                self.filter_pass(encoder, dest, source, Some(&blurred), args)?;
            }
            LayerFilter::ColorMatrix(matrix) => {
                let mut coefficients = [0.0_f32; 20];
                for (c, v) in coefficients.iter_mut().zip(matrix.0) {
                    *c = v as f32;
                }
                let args = Pass {
                    mode: MODE_MATRIX,
                    matrix: coefficients,
                    ..Pass::default()
                };
                self.filter_pass(encoder, dest, source, None, args)?;
            }
            // Graphs and shaders are not run by this executor yet: their input
            // passes through (the CPU's rule for these filters).
            LayerFilter::Graph(_) | LayerFilter::Shader(_) => copy(encoder, source, dest),
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
        let params = super::exec::uniform(&self.device, &self.queue, words.bytes());
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

    /// The premultiplied colour `color` where `source` is opaque (the drop shadow's silhouette).
    fn shadow(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        color: kivact_types::color::Rgba,
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

    /// A new texture holding `src` gaussian-blurred by `sigma_px` (design §8.8.1).
    fn blur(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::Texture,
        sigma_px: f64,
    ) -> Result<wgpu::Texture, ComposeError> {
        let (w, h) = (src.width(), src.height());
        let first = targets::target(&self.device, (w, h));
        copy(encoder, src, &first);
        if sigma_px < 0.5 {
            return Ok(first);
        }
        let mut sigma = sigma_px;
        let mut levels = 0;
        while sigma > BLUR_LIMIT && levels < BLUR_LEVELS {
            sigma /= 2.0;
            levels += 1;
        }
        let mut chain = vec![first];
        for _ in 0..levels {
            let prev = chain
                .last()
                .expect("invariant: the chain starts with a texture");
            let next = targets::target(&self.device, half(prev.width(), prev.height()));
            let args = Pass {
                out: size_f(&next),
                src: size_f(prev),
                mode: MODE_DOWNSCALE,
                ..Pass::default()
            };
            self.filter_pass(encoder, &next, prev, None, args)?;
            chain.push(next);
        }
        let small = chain.last().expect("invariant: the chain is not empty");
        let (radius, weights) = gaussian(sigma);
        let blurred = self.blur_separable(encoder, small, radius, &weights)?;
        let current = self.upscale_chain(encoder, &chain, blurred)?;
        debug_assert_eq!((current.width(), current.height()), (w, h));
        Ok(current)
    }

    /// Blur `small` horizontally, then vertically, with the gaussian `weights` of `radius`.
    fn blur_separable(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        small: &wgpu::Texture,
        radius: u32,
        weights: &[u8],
    ) -> Result<wgpu::Texture, ComposeError> {
        let weight_buffer = buffer(
            &self.device,
            &self.queue,
            wgpu::BufferUsages::STORAGE,
            weights,
        );
        let horizontal = targets::target(&self.device, (small.width(), small.height()));
        let blur_x = Pass {
            out: size_f(&horizontal),
            src: size_f(small),
            dir: (1.0, 0.0),
            radius,
            mode: MODE_BLUR,
            ..Pass::default()
        };
        self.filter_pass_with(
            encoder,
            &horizontal,
            small,
            None,
            blur_x,
            Some(&weight_buffer),
        )?;
        let vertical = targets::target(&self.device, (small.width(), small.height()));
        let blur_y = Pass {
            out: size_f(&vertical),
            src: size_f(&horizontal),
            dir: (0.0, 1.0),
            radius,
            mode: MODE_BLUR,
            ..Pass::default()
        };
        self.filter_pass_with(
            encoder,
            &vertical,
            &horizontal,
            None,
            blur_y,
            Some(&weight_buffer),
        )?;
        Ok(vertical)
    }

    /// Upscale `current` back up the levels of `chain`, from its smallest level to its largest.
    fn upscale_chain(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        chain: &[wgpu::Texture],
        mut current: wgpu::Texture,
    ) -> Result<wgpu::Texture, ComposeError> {
        for level in chain.iter().rev().skip(1) {
            let up = targets::target(&self.device, (level.width(), level.height()));
            let args = Pass {
                out: size_f(&up),
                src: size_f(&current),
                mode: MODE_UPSCALE,
                ..Pass::default()
            };
            self.filter_pass(encoder, &up, &current, None, args)?;
            current = up;
        }
        Ok(current)
    }
}

/// Copy all of `src` into `dst` (the same size and format).
fn copy(encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture, dst: &wgpu::Texture) {
    encoder.copy_texture_to_texture(src.as_image_copy(), dst.as_image_copy(), src.size());
}

/// The size of a texture halved (rounded up), as one downscale level.
fn half(w: u32, h: u32) -> (u32, u32) {
    (w.div_ceil(2), h.div_ceil(2))
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
