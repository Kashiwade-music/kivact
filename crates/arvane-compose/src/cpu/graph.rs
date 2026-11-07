// Source: resvg 0.46.0, src/filter/mod.rs (Copyright 2018 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// The primitive driver, the color space conversions and the sRGB tables are
// resvg's. Changes: the graph is kivact's FilterGraph (indices instead of
// names, no usvg types); the region is the whole target; GaussianBlur and
// DropShadow use design §8.8.1 and §8.8.2 instead of resvg's box and IIR
// blurs; the pixel type is the compositor's premultiplied pixmap; Tile samples
// the tile rectangle with nearest filtering; Image stretches its pixels into
// its rectangle.
// The resvg files component_transfer.rs, composite.rs, convolve_matrix.rs,
// displacement_map.rs, lighting.rs, morphology.rs and turbulence.rs are
// ported under graph/ with their own headers.

//! Role: Runs a filter graph of SVG primitives on the CPU (design §8.8.4).

mod component_transfer;
mod composite;
mod convolve_matrix;
mod displacement_map;
mod lighting;
mod morphology;
mod turbulence;

use std::rc::Rc;

use kivact_types::Rgba;
use kivact_types::effect::{
    CompositeOp, FilterColorSpace, FilterGraph, FilterInput, Light, PrimitiveOp, TurbulenceKind,
};
use kivact_types::geometry::{Point, Rect};
use kivact_types::pixel::{BlendMode, Pixels};
use kivact_types::render::Composite;

use super::blend::combine;
use super::blur::gaussian;
use super::filters::{blur_sigmas, color_matrix, drop_shadow};
use super::surface::{Pixmap, Premul, sample_bilinear, sample_nearest};
use crate::ComposeError;

#[rustfmt::skip]
const SRGB_TO_LINEAR_RGB_TABLE: &[u8; 256] = &[
    0,   0,   0,   0,   0,   0,  0,    1,   1,   1,   1,   1,   1,   1,   1,   1,
    1,   1,   2,   2,   2,   2,  2,    2,   2,   2,   3,   3,   3,   3,   3,   3,
    4,   4,   4,   4,   4,   5,  5,    5,   5,   6,   6,   6,   6,   7,   7,   7,
    8,   8,   8,   8,   9,   9,  9,   10,  10,  10,  11,  11,  12,  12,  12,  13,
    13,  13,  14,  14,  15,  15,  16,  16,  17,  17,  17,  18,  18,  19,  19,  20,
    20,  21,  22,  22,  23,  23,  24,  24,  25,  25,  26,  27,  27,  28,  29,  29,
    30,  30,  31,  32,  32,  33,  34,  35,  35,  36,  37,  37,  38,  39,  40,  41,
    41,  42,  43,  44,  45,  45,  46,  47,  48,  49,  50,  51,  51,  52,  53,  54,
    55,  56,  57,  58,  59,  60,  61,  62,  63,  64,  65,  66,  67,  68,  69,  70,
    71,  72,  73,  74,  76,  77,  78,  79,  80,  81,  82,  84,  85,  86,  87,  88,
    90,  91,  92,  93,  95,  96,  97,  99, 100, 101, 103, 104, 105, 107, 108, 109,
    111, 112, 114, 115, 116, 118, 119, 121, 122, 124, 125, 127, 128, 130, 131, 133,
    134, 136, 138, 139, 141, 142, 144, 146, 147, 149, 151, 152, 154, 156, 157, 159,
    161, 163, 164, 166, 168, 170, 171, 173, 175, 177, 179, 181, 183, 184, 186, 188,
    190, 192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212, 214, 216, 218, 220,
    222, 224, 226, 229, 231, 233, 235, 237, 239, 242, 244, 246, 248, 250, 253, 255,
];

/// Precomputed LinearRGB to sRGB table.
///
/// Since we are storing the result in `u8`, there is no need to compute those
/// values each time. Mainly because it's very expensive.
///
/// ```text
/// if (C_lin <= 0.0031308)
///     C_srgb = C_lin * 12.92;
/// else
///     C_srgb = 1.055 * pow(C_lin, 1.0 / 2.4) - 0.055;
/// ```
///
/// Thanks to librsvg for the idea.
#[rustfmt::skip]
const LINEAR_RGB_TO_SRGB_TABLE: &[u8; 256] = &[
    0,  13,  22,  28,  34,  38,  42,  46,  50,  53,  56,  59,  61,  64,  66,  69,
    71,  73,  75,  77,  79,  81,  83,  85,  86,  88,  90,  92,  93,  95,  96,  98,
    99, 101, 102, 104, 105, 106, 108, 109, 110, 112, 113, 114, 115, 117, 118, 119,
    120, 121, 122, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136,
    137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 148, 149, 150, 151,
    152, 153, 154, 155, 155, 156, 157, 158, 159, 159, 160, 161, 162, 163, 163, 164,
    165, 166, 167, 167, 168, 169, 170, 170, 171, 172, 173, 173, 174, 175, 175, 176,
    177, 178, 178, 179, 180, 180, 181, 182, 182, 183, 184, 185, 185, 186, 187, 187,
    188, 189, 189, 190, 190, 191, 192, 192, 193, 194, 194, 195, 196, 196, 197, 197,
    198, 199, 199, 200, 200, 201, 202, 202, 203, 203, 204, 205, 205, 206, 206, 207,
    208, 208, 209, 209, 210, 210, 211, 212, 212, 213, 213, 214, 214, 215, 215, 216,
    216, 217, 218, 218, 219, 219, 220, 220, 221, 221, 222, 222, 223, 223, 224, 224,
    225, 226, 226, 227, 227, 228, 228, 229, 229, 230, 230, 231, 231, 232, 232, 233,
    233, 234, 234, 235, 235, 236, 236, 237, 237, 238, 238, 238, 239, 239, 240, 240,
    241, 241, 242, 242, 243, 243, 244, 244, 245, 245, 246, 246, 246, 247, 247, 248,
    248, 249, 249, 250, 250, 251, 251, 251, 252, 252, 253, 253, 254, 254, 255, 255,
];

/// A primitive's result: its pixels and the color space they are in.
#[derive(Clone)]
struct Image {
    /// The pixels, shared between the results that read them.
    pixmap: Rc<Pixmap>,
    /// The color space of the pixels.
    cs: FilterColorSpace,
}

impl Image {
    /// `pixmap` in color space `cs`.
    fn new(pixmap: Pixmap, cs: FilterColorSpace) -> Self {
        Image {
            pixmap: Rc::new(pixmap),
            cs,
        }
    }

    /// The pixels, copied only when another result still holds them.
    fn take(self) -> Pixmap {
        Rc::try_unwrap(self.pixmap).unwrap_or_else(|shared| (*shared).clone())
    }
}

/// What a primitive reads: the source, the earlier results and the mapping to px.
struct Env<'a> {
    /// The layer's source graphic, premultiplied sRGB.
    source: &'a Rc<Pixmap>,
    /// The results of the earlier primitives.
    results: &'a [Image],
    /// Device px per layer unit.
    ppu: f64,
    /// The layer point at px (0, 0).
    origin: Point,
}

/// The filtered `input` of `graph`; `ppu` px per layer unit and `origin` the layer point at px (0, 0).
pub(crate) fn apply(
    graph: &FilterGraph<Pixels>,
    input: &Pixmap,
    ppu: f64,
    origin: Point,
) -> Result<Pixmap, ComposeError> {
    graph.validate().map_err(ComposeError::InvalidComposition)?;
    let source = Rc::new(input.clone());
    let mut results: Vec<Image> = Vec::with_capacity(graph.primitives.len());
    for primitive in &graph.primitives {
        let cs = primitive.color_space.unwrap_or(graph.color_space);
        let env = Env {
            source: &source,
            results: &results,
            ppu,
            origin,
        };
        let image = run(&primitive.op, cs, &env)?;
        results.push(image);
    }
    let last = results
        .pop()
        .ok_or_else(|| ComposeError::InvalidComposition("filter graph has no primitives".into()))?;
    Ok(into_space(last, FilterColorSpace::Srgb).take())
}

/// The image `input` names: the source, its alpha, or an earlier result.
fn fetch(input: FilterInput, env: &Env<'_>) -> Result<Image, ComposeError> {
    match input {
        FilterInput::Source => Ok(Image {
            pixmap: Rc::clone(env.source),
            cs: FilterColorSpace::Srgb,
        }),
        FilterInput::SourceAlpha => {
            let mut px = (**env.source).clone();
            for p in px.data.chunks_exact_mut(4) {
                p[0] = 0;
                p[1] = 0;
                p[2] = 0;
            }
            Ok(Image::new(px, FilterColorSpace::Srgb))
        }
        FilterInput::Result(i) => env.results.get(i as usize).cloned().ok_or_else(|| {
            ComposeError::InvalidComposition(format!("filter result {i} is not computed yet"))
        }),
    }
}

/// `img` in color space `cs`, converted when it differs.
fn into_space(img: Image, cs: FilterColorSpace) -> Image {
    if img.cs == cs {
        return img;
    }
    let mut px = img.take();
    let table = match cs {
        FilterColorSpace::Srgb => &LINEAR_RGB_TO_SRGB_TABLE,
        FilterColorSpace::LinearRgb => &SRGB_TO_LINEAR_RGB_TABLE,
    };
    demultiply(&mut px);
    for p in px.data.chunks_exact_mut(4) {
        for c in &mut p[..3] {
            *c = table[usize::from(*c)];
        }
    }
    multiply(&mut px);
    Image::new(px, cs)
}

/// Unpremultiplies the bytes of `px` (resvg's rounding).
fn demultiply(px: &mut Pixmap) {
    for p in px.data.chunks_exact_mut(4) {
        let a = f32::from(p[3]) / 255.0;
        for c in &mut p[..3] {
            *c = (f32::from(*c) / a + 0.5) as u8;
        }
    }
}

/// Premultiplies the bytes of `px` (resvg's rounding).
fn multiply(px: &mut Pixmap) {
    for p in px.data.chunks_exact_mut(4) {
        let a = f32::from(p[3]) / 255.0;
        for c in &mut p[..3] {
            *c = (f32::from(*c) * a + 0.5) as u8;
        }
    }
}

/// Runs one primitive in color space `cs`.
fn run(
    op: &PrimitiveOp<Pixels>,
    cs: FilterColorSpace,
    env: &Env<'_>,
) -> Result<Image, ComposeError> {
    match op {
        PrimitiveOp::Blend { top, bottom, mode } => {
            let mut dst = into_space(fetch(*bottom, env)?, cs).take();
            let src = into_space(fetch(*top, env)?, cs);
            draw(&mut dst, &src.pixmap, *mode, Composite::SourceOver);
            Ok(Image::new(dst, cs))
        }
        PrimitiveOp::ColorMatrix { input, matrix } => {
            let img = into_space(fetch(*input, env)?, cs);
            Ok(Image::new(color_matrix(&img.pixmap, matrix), cs))
        }
        PrimitiveOp::ComponentTransfer { input, r, g, b, a } => {
            let mut px = into_space(fetch(*input, env)?, cs).take();
            let funcs = component_transfer::Transfers { r, g, b, a };
            component_transfer::apply(&funcs, &mut px);
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::Composite { top, bottom, op } => composite_op(*top, *bottom, *op, cs, env),
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
            let mut px = into_space(fetch(*input, env)?, cs).take();
            if *preserve_alpha {
                demultiply(&mut px);
            }
            let conv = convolve_matrix::Convolution {
                order: *order,
                kernel,
                divisor: *divisor,
                bias: *bias,
                target: *target,
                edge: *edge,
                preserve_alpha: *preserve_alpha,
            };
            convolve_matrix::apply(&conv, &mut px);
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::DiffuseLighting {
            input,
            surface_scale,
            diffuse_constant,
            color,
            light,
        } => {
            let img = fetch(*input, env)?;
            let kind = lighting::Kind::Diffuse {
                constant: *diffuse_constant as f32,
            };
            let px = lighting::apply(
                kind,
                *surface_scale as f32,
                lighting_color(*color),
                light_px(light, env),
                &img.pixmap,
            );
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::SpecularLighting {
            input,
            surface_scale,
            specular_constant,
            specular_exponent,
            color,
            light,
        } => {
            let img = fetch(*input, env)?;
            let kind = lighting::Kind::Specular {
                constant: *specular_constant as f32,
                exponent: *specular_exponent as f32,
            };
            let px = lighting::apply(
                kind,
                *surface_scale as f32,
                lighting_color(*color),
                light_px(light, env),
                &img.pixmap,
            );
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::DisplacementMap {
            input,
            map,
            scale,
            x,
            y,
        } => {
            let src = into_space(fetch(*input, env)?, cs);
            let map = into_space(fetch(*map, env)?, cs);
            let s = (scale * env.ppu) as f32;
            let px = displacement_map::apply(&src.pixmap, &map.pixmap, (s, s), (*x, *y));
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::DropShadow {
            input,
            offset,
            std_dev,
            color,
        } => {
            let img = into_space(fetch(*input, env)?, cs);
            let tint = color_in(*color, cs);
            let shifted = (offset.x * env.ppu, offset.y * env.ppu);
            let sigmas = (std_dev.0 * env.ppu, std_dev.1 * env.ppu);
            let px = drop_shadow(&img.pixmap, shifted, sigmas, tint);
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::Flood { color } => {
            let mut px = Pixmap::new(env.source.size().0, env.source.size().1);
            let premul = premultiplied(*color);
            let (w, h) = px.size();
            for y in 0..h {
                for x in 0..w {
                    px.set(x, y, premul);
                }
            }
            Ok(Image::new(px, FilterColorSpace::Srgb))
        }
        PrimitiveOp::GaussianBlur { input, std_dev, .. } => {
            let img = into_space(fetch(*input, env)?, cs);
            let (sx, sy) = (std_dev.0 * env.ppu, std_dev.1 * env.ppu);
            let px = match blur_sigmas(sx, sy) {
                Some((sx, sy)) => gaussian(&img.pixmap, sx, sy),
                None => (*img.pixmap).clone(),
            };
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::Image { image, rect } => Ok(Image::new(
            stretch(image, *rect, env),
            FilterColorSpace::Srgb,
        )),
        PrimitiveOp::Merge { inputs } => {
            let mut dst = Pixmap::new(env.source.size().0, env.source.size().1);
            for input in inputs {
                let src = into_space(fetch(*input, env)?, cs);
                draw(
                    &mut dst,
                    &src.pixmap,
                    BlendMode::Normal,
                    Composite::SourceOver,
                );
            }
            Ok(Image::new(dst, cs))
        }
        PrimitiveOp::Morphology { input, op, radius } => {
            let mut px = into_space(fetch(*input, env)?, cs).take();
            let (rx, ry) = (radius.0 * env.ppu, radius.1 * env.ppu);
            if rx > 0.0 && ry > 0.0 {
                morphology::apply(*op, rx, ry, &mut px);
            } else {
                px.clear(None);
            }
            Ok(Image::new(px, cs))
        }
        PrimitiveOp::Offset { input, offset } => {
            let img = fetch(*input, env)?;
            let dx = (offset.x * env.ppu) as i64;
            let dy = (offset.y * env.ppu) as i64;
            if dx == 0 && dy == 0 {
                return Ok(img);
            }
            let cs_in = img.cs;
            Ok(Image::new(shifted(&img.pixmap, dx, dy), cs_in))
        }
        PrimitiveOp::Tile { input, tile } => {
            let img = fetch(*input, env)?;
            let cs_in = img.cs;
            Ok(Image::new(tiled(&img.pixmap, *tile, env), cs_in))
        }
        PrimitiveOp::Turbulence {
            base_frequency,
            octaves,
            seed,
            kind,
            stitch,
        } => {
            let (w, h) = env.source.size();
            let params = turbulence::Params {
                offset_x: env.origin.x * env.ppu,
                offset_y: env.origin.y * env.ppu,
                sx: 1.0,
                sy: 1.0,
                base_x: base_frequency.0 / env.ppu,
                base_y: base_frequency.1 / env.ppu,
                octaves: *octaves,
                seed: seed.trunc() as i32,
                stitch: if *stitch {
                    turbulence::Stitching::On
                } else {
                    turbulence::Stitching::Off
                },
                kind: if *kind == TurbulenceKind::FractalNoise {
                    turbulence::NoiseKind::Fractal
                } else {
                    turbulence::NoiseKind::Turbulence
                },
            };
            Ok(Image::new(turbulence::apply(&params, w, h), cs))
        }
    }
}

/// feComposite: the arithmetic operator on the two inputs, or the Porter–Duff operator with `top` over `bottom`.
fn composite_op(
    top: FilterInput,
    bottom: FilterInput,
    op: CompositeOp,
    cs: FilterColorSpace,
    env: &Env<'_>,
) -> Result<Image, ComposeError> {
    let first = into_space(fetch(top, env)?, cs);
    let second = into_space(fetch(bottom, env)?, cs);
    if let CompositeOp::Arithmetic { k1, k2, k3, k4 } = op {
        let k = composite::Coefficients {
            k1: k1 as f32,
            k2: k2 as f32,
            k3: k3 as f32,
            k4: k4 as f32,
        };
        return Ok(Image::new(
            composite::arithmetic(k, &first.pixmap, &second.pixmap),
            cs,
        ));
    }
    let mut dst = second.take();
    let porter = match op {
        CompositeOp::Over | CompositeOp::Arithmetic { .. } => Composite::SourceOver,
        CompositeOp::In => Composite::SourceIn,
        CompositeOp::Out => Composite::SourceOut,
        CompositeOp::Atop => Composite::SourceAtop,
        CompositeOp::Xor => Composite::Xor,
        CompositeOp::Lighter => Composite::Lighter,
    };
    draw(&mut dst, &first.pixmap, BlendMode::Normal, porter);
    Ok(Image::new(dst, cs))
}

/// Draws `src` over `dst` pixel by pixel with `blend` and `op`; a transparent source pixel leaves `dst` as it is under SourceOver.
fn draw(dst: &mut Pixmap, src: &Pixmap, blend: BlendMode, op: Composite) {
    let (w, h) = dst.size();
    let skip_transparent = blend == BlendMode::Normal && op == Composite::SourceOver;
    for y in 0..h {
        for x in 0..w {
            let s = src.get(x, y);
            if skip_transparent && s[3] <= 0.0 {
                continue;
            }
            let d = dst.get(x, y);
            dst.set(x, y, combine(s, d, blend, op));
        }
    }
}

/// `src` moved by whole pixels (`dx`, `dy`), transparent where it does not reach.
fn shifted(src: &Pixmap, dx: i64, dy: i64) -> Pixmap {
    let (w, h) = src.size();
    let mut out = Pixmap::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let sx = i64::from(x) - dx;
            let sy = i64::from(y) - dy;
            if (0..i64::from(w)).contains(&sx) && (0..i64::from(h)).contains(&sy) {
                out.set(x, y, src.get(sx as u32, sy as u32));
            }
        }
    }
    out
}

/// The pixels of `src` repeated over the tile rectangle `tile` (layer space).
fn tiled(src: &Pixmap, tile: Rect, env: &Env<'_>) -> Pixmap {
    let (w, h) = src.size();
    let mut out = Pixmap::new(w, h);
    let (tx, ty) = layer_to_px(tile.x, tile.y, env);
    let (tw, th) = (tile.width * env.ppu, tile.height * env.ppu);
    if tw <= 0.0 || th <= 0.0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let px = f64::from(x) + 0.5;
            let py = f64::from(y) + 0.5;
            let u = (px - tx).rem_euclid(tw);
            let v = (py - ty).rem_euclid(th);
            out.set(x, y, sample_nearest(src, Point::new(tx + u, ty + v)));
        }
    }
    out
}

/// The pixels of `image` (straight alpha) stretched over `rect` (layer space) and nothing else.
fn stretch(image: &Pixels, rect: Rect, env: &Env<'_>) -> Pixmap {
    let (w, h) = env.source.size();
    let mut out = Pixmap::new(w, h);
    let source = straight_to_premul(image);
    let (rx, ry) = layer_to_px(rect.x, rect.y, env);
    let (rw, rh) = (rect.width * env.ppu, rect.height * env.ppu);
    if rw <= 0.0 || rh <= 0.0 {
        return out;
    }
    for y in 0..h {
        for x in 0..w {
            let p = Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
            if p.x < rx || p.y < ry || p.x >= rx + rw || p.y >= ry + rh {
                continue;
            }
            let u = (p.x - rx) / rw * f64::from(image.width);
            let v = (p.y - ry) / rh * f64::from(image.height);
            out.set(x, y, sample_bilinear(&source, Point::new(u, v)));
        }
    }
    out
}

/// A layer point in px.
fn layer_to_px(x: f64, y: f64, env: &Env<'_>) -> (f64, f64) {
    ((x - env.origin.x) * env.ppu, (y - env.origin.y) * env.ppu)
}

/// Straight-alpha pixels as a premultiplied pixmap.
fn straight_to_premul(image: &Pixels) -> Pixmap {
    let mut out = Pixmap::new(image.width, image.height);
    let texels = image.width as usize * image.height as usize;
    for (i, p) in image.rgba.chunks_exact(4).take(texels).enumerate() {
        let a = f32::from(p[3]) / 255.0;
        let color: Premul = [
            f32::from(p[0]) / 255.0 * a,
            f32::from(p[1]) / 255.0 * a,
            f32::from(p[2]) / 255.0 * a,
            a,
        ];
        let (x, y) = (i as u32 % image.width, i as u32 / image.width);
        out.set(x, y, color);
    }
    out
}

/// A color as a premultiplied pixel.
fn premultiplied(c: Rgba) -> Premul {
    let a = c.a as f32;
    [c.r as f32 * a, c.g as f32 * a, c.b as f32 * a, a]
}

/// A shadow color in color space `cs` (sRGB unchanged; linear through the table).
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

/// The lighting color as 0..=255 channels.
fn lighting_color(c: Rgba) -> [f32; 3] {
    [c.r as f32 * 255.0, c.g as f32 * 255.0, c.b as f32 * 255.0]
}

/// A light source with its positions in px (z scaled by the px per unit).
fn light_px(light: &Light, env: &Env<'_>) -> lighting::LightPx {
    let to_px = |p: [f64; 3]| {
        let (x, y) = layer_to_px(p[0], p[1], env);
        [x as f32, y as f32, (p[2] * env.ppu) as f32]
    };
    match light {
        Light::Distant {
            azimuth_deg,
            elevation_deg,
        } => lighting::LightPx::Distant {
            azimuth: *azimuth_deg as f32,
            elevation: *elevation_deg as f32,
        },
        Light::Point { position } => lighting::LightPx::Point {
            position: to_px(*position),
        },
        Light::Spot {
            position,
            points_at,
            exponent,
            cone_deg,
        } => lighting::LightPx::Spot {
            position: to_px(*position),
            points_at: to_px(*points_at),
            exponent: *exponent as f32,
            cone: cone_deg.map(|c| c as f32),
        },
    }
}
