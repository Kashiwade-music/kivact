// Source: resvg 0.46.0, src/filter/turbulence.rs (Copyright 2020 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Changes: the parameters are plain values; the output is written as a
// premultiplied pixmap; the stitch and noise routines are unchanged.

//! Role: feTurbulence on the CPU (Perlin noise by the SVG reference algorithm).

use crate::cpu::surface::Pixmap;
use crate::tables::{Tables, init};

const BM: i32 = 0xff;
const PERLIN_N: i32 = 0x1000;

/// The parameters of one turbulence fill.
pub(crate) struct Params {
    /// Added to the pixel x before scaling (px).
    pub(crate) offset_x: f64,
    /// Added to the pixel y before scaling (px).
    pub(crate) offset_y: f64,
    /// Scale from pixels to noise space.
    pub(crate) sx: f64,
    /// Scale from pixels to noise space.
    pub(crate) sy: f64,
    /// Base frequency along x.
    pub(crate) base_x: f64,
    /// Base frequency along y.
    pub(crate) base_y: f64,
    /// Octaves summed.
    pub(crate) octaves: u32,
    /// Seed (truncated to an integer).
    pub(crate) seed: i32,
    /// Whether the frequencies stitch across the tile.
    pub(crate) stitch: Stitching,
    /// Fractal noise or turbulence.
    pub(crate) kind: NoiseKind,
}

/// Whether a turbulence tile stitches its frequencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stitching {
    /// Frequencies are used as given.
    Off,
    /// Frequencies are adjusted so tile borders match.
    On,
}

/// The kind of noise a turbulence fill makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NoiseKind {
    /// Fractal noise.
    Fractal,
    /// Turbulence: absolute values of the noise.
    Turbulence,
}

/// Stitch information of a tile for one octave.
#[derive(Clone, Copy)]
struct StitchInfo {
    /// How much to subtract to wrap for stitching.
    width: i32,
    /// As `width`, vertically.
    height: i32,
    /// Minimum value to wrap.
    wrap_x: i32,
    /// As `wrap_x`, vertically.
    wrap_y: i32,
}

/// A `w` × `h` pixmap of the turbulence noise (premultiplied).
pub(crate) fn apply(p: &Params, w: u32, h: u32) -> Pixmap {
    let tables = init(p.seed);
    let mut out = Pixmap::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut c = [0.0_f32; 4];
            for (channel, value) in c.iter_mut().enumerate() {
                let tx = (f64::from(x) + p.offset_x) / p.sx;
                let ty = (f64::from(y) + p.offset_y) / p.sy;
                let n = turbulence(
                    channel,
                    tx,
                    ty,
                    (f64::from(x), f64::from(y)),
                    (f64::from(w), f64::from(h)),
                    p,
                    &tables,
                );
                let n = if p.kind == NoiseKind::Fractal {
                    (n * 255.0 + 255.0) / 2.0
                } else {
                    n * 255.0
                };
                *value = ((n as f32).clamp(0.0, 255.0) / 255.0).clamp(0.0, 1.0);
            }
            let a = c[3];
            out.set(x, y, [c[0] * a, c[1] * a, c[2] * a, a]);
        }
    }
    out
}

/// One channel's turbulence at the noise point (`x`, `y`) for the tile at `tile` of size `size`.
fn turbulence(
    channel: usize,
    mut x: f64,
    mut y: f64,
    tile: (f64, f64),
    size: (f64, f64),
    p: &Params,
    t: &Tables,
) -> f64 {
    let (mut base_x, mut base_y) = (p.base_x, p.base_y);
    let mut stitch = if p.stitch == Stitching::On {
        if !is_zero(base_x) {
            let lo = (size.0 * base_x).floor() / size.0;
            let hi = (size.0 * base_x).ceil() / size.0;
            base_x = if base_x / lo < hi / base_x { lo } else { hi };
        }
        if !is_zero(base_y) {
            let lo = (size.1 * base_y).floor() / size.1;
            let hi = (size.1 * base_y).ceil() / size.1;
            base_y = if base_y / lo < hi / base_y { lo } else { hi };
        }
        let width = (size.0 * base_x + 0.5) as i32;
        let height = (size.1 * base_y + 0.5) as i32;
        let wrap_x = (tile.0 * base_x + f64::from(PERLIN_N) + f64::from(width)) as i32;
        let wrap_y = (tile.1 * base_y + f64::from(PERLIN_N) + f64::from(height)) as i32;
        Some(StitchInfo {
            width,
            height,
            wrap_x,
            wrap_y,
        })
    } else {
        None
    };
    let mut sum = 0.0;
    x *= base_x;
    y *= base_y;
    let mut ratio = 1.0;
    for _ in 0..p.octaves {
        let n = noise2(channel, x, y, t, stitch);
        let term = if p.kind == NoiseKind::Fractal {
            n
        } else {
            n.abs()
        };
        sum += term / ratio;
        x *= 2.0;
        y *= 2.0;
        ratio *= 2.0;
        if let Some(s) = stitch.as_mut() {
            s.width *= 2;
            s.wrap_x = 2 * s.wrap_x - PERLIN_N;
            s.height *= 2;
            s.wrap_y = 2 * s.wrap_y - PERLIN_N;
        }
    }
    sum
}

/// Whether `v` is zero within four ulps.
fn is_zero(v: f64) -> bool {
    v.abs() <= f64::EPSILON * 4.0
}

/// Gradient noise at (`x`, `y`) for one channel.
fn noise2(channel: usize, x: f64, y: f64, t: &Tables, stitch: Option<StitchInfo>) -> f64 {
    let lattice = &t.lattice;
    let tt = x + f64::from(PERLIN_N);
    let mut bx0 = tt as i32;
    let mut bx1 = bx0 + 1;
    let rx0 = tt - tt as i64 as f64;
    let rx1 = rx0 - 1.0;
    let tt = y + f64::from(PERLIN_N);
    let mut by0 = tt as i32;
    let mut by1 = by0 + 1;
    let ry0 = tt - tt as i64 as f64;
    let ry1 = ry0 - 1.0;
    if let Some(info) = stitch {
        if bx0 >= info.wrap_x {
            bx0 -= info.width;
        }
        if bx1 >= info.wrap_x {
            bx1 -= info.width;
        }
        if by0 >= info.wrap_y {
            by0 -= info.height;
        }
        if by1 >= info.wrap_y {
            by1 -= info.height;
        }
    }
    bx0 &= BM;
    bx1 &= BM;
    by0 &= BM;
    by1 &= BM;
    let i = lattice[bx0 as usize];
    let j = lattice[bx1 as usize];
    let b00 = lattice[i + by0 as usize];
    let b10 = lattice[j + by0 as usize];
    let b01 = lattice[i + by1 as usize];
    let b11 = lattice[j + by1 as usize];
    let sx = s_curve(rx0);
    let sy = s_curve(ry0);
    let g = &t.gradient[channel];
    let q = g[b00];
    let u = rx0 * q[0] + ry0 * q[1];
    let q = g[b10];
    let v = rx1 * q[0] + ry0 * q[1];
    let a = lerp(sx, u, v);
    let q = g[b01];
    let u = rx0 * q[0] + ry1 * q[1];
    let q = g[b11];
    let v = rx1 * q[0] + ry1 * q[1];
    let b = lerp(sx, u, v);
    lerp(sy, a, b)
}

/// The smoothstep `t² (3 − 2t)`.
fn s_curve(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation from `a` to `b` by `t`.
fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}
