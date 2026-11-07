//! Role: Blend modes (W3C Compositing 1) and Porter–Duff operators on premultiplied colors (design §8.7).

use kivact_types::pixel::BlendMode;
use kivact_types::render::Composite;

use super::surface::Premul;

/// Combines the source `src` (already multiplied by coverage and opacity) with the destination `dst`.
pub(crate) fn combine(src: Premul, dst: Premul, blend: BlendMode, op: Composite) -> Premul {
    match blend {
        BlendMode::PlusLighter => std::array::from_fn(|i| (src[i] + dst[i]).min(1.0)),
        BlendMode::Normal => porter_duff(src, dst, op),
        _ => blended(src, dst, blend),
    }
}

/// `result = Fa·src + Fb·dst` for the operator, with the factors of design §8.7.
fn porter_duff(src: Premul, dst: Premul, op: Composite) -> Premul {
    let (as_, ab) = (src[3], dst[3]);
    let (fa, fb) = match op {
        Composite::SourceOver => (1.0, 1.0 - as_),
        Composite::SourceIn => (ab, 0.0),
        Composite::SourceOut => (1.0 - ab, 0.0),
        Composite::SourceAtop => (ab, 1.0 - as_),
        Composite::DestinationOver => (1.0 - ab, 1.0),
        Composite::DestinationIn => (0.0, as_),
        Composite::DestinationOut => (0.0, 1.0 - as_),
        Composite::DestinationAtop => (1.0 - ab, as_),
        Composite::Lighter => (1.0, 1.0),
        Composite::Copy => (1.0, 0.0),
        Composite::Xor => (1.0 - ab, 1.0 - as_),
    };
    let out: Premul = std::array::from_fn(|i| fa * src[i] + fb * dst[i]);
    if op == Composite::Lighter {
        out.map(|v| v.min(1.0))
    } else {
        out
    }
}

/// `co = cs(1 − αb) + cb(1 − αs) + αs·αb·B(Cb, Cs)` and `αo = αs + αb(1 − αs)`.
fn blended(src: Premul, dst: Premul, blend: BlendMode) -> Premul {
    let (as_, ab) = (src[3], dst[3]);
    let cs = unpremultiply(src);
    let cb = unpremultiply(dst);
    let b = blend_color(blend, cb, cs);
    let mut out = [0.0_f32; 4];
    for i in 0..3 {
        out[i] = src[i] * (1.0 - ab) + dst[i] * (1.0 - as_) + as_ * ab * b[i];
    }
    out[3] = as_ + ab * (1.0 - as_);
    out
}

/// The unpremultiplied RGB of `c` (zero where alpha is zero).
fn unpremultiply(c: Premul) -> [f32; 3] {
    if c[3] <= 0.0 {
        return [0.0; 3];
    }
    [c[0] / c[3], c[1] / c[3], c[2] / c[3]]
}

/// B(Cb, Cs) for each channel of an unpremultiplied backdrop and source.
fn blend_color(mode: BlendMode, cb: [f32; 3], cs: [f32; 3]) -> [f32; 3] {
    match mode {
        BlendMode::Hue => set_lum(set_sat(cs, sat(cb)), lum(cb)),
        BlendMode::Saturation => set_lum(set_sat(cb, sat(cs)), lum(cb)),
        BlendMode::Color => set_lum(cs, lum(cb)),
        BlendMode::Luminosity => set_lum(cb, lum(cs)),
        _ => std::array::from_fn(|i| separable(mode, cb[i], cs[i])),
    }
}

/// The separable blend functions of design §8.7 for one channel.
fn separable(mode: BlendMode, cb: f32, cs: f32) -> f32 {
    match mode {
        BlendMode::Multiply => cb * cs,
        BlendMode::Screen => cb + cs - cb * cs,
        BlendMode::Overlay => hard_light(cs, cb),
        BlendMode::Darken => cb.min(cs),
        BlendMode::Lighten => cb.max(cs),
        BlendMode::ColorDodge => {
            if cb == 0.0 {
                0.0
            } else if cs >= 1.0 {
                1.0
            } else {
                (cb / (1.0 - cs)).min(1.0)
            }
        }
        BlendMode::ColorBurn => {
            if cb >= 1.0 {
                1.0
            } else if cs <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - cb) / cs).min(1.0)
            }
        }
        BlendMode::HardLight => hard_light(cb, cs),
        BlendMode::SoftLight => soft_light(cb, cs),
        BlendMode::Difference => (cb - cs).abs(),
        BlendMode::Exclusion => cb + cs - 2.0 * cb * cs,
        _ => cs,
    }
}

/// HardLight(Cb, Cs) of W3C Compositing 1.
fn hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        cb * 2.0 * cs
    } else {
        let s = 2.0 * cs - 1.0;
        cb + s - cb * s
    }
}

/// SoftLight(Cb, Cs) of W3C Compositing 1.
fn soft_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
    } else {
        let d = if cb <= 0.25 {
            ((16.0 * cb - 12.0) * cb + 4.0) * cb
        } else {
            cb.sqrt()
        };
        cb + (2.0 * cs - 1.0) * (d - cb)
    }
}

/// Lum(C) = 0.3R + 0.59G + 0.11B.
fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

/// Sat(C) = max − min.
fn sat(c: [f32; 3]) -> f32 {
    c.iter().copied().fold(f32::MIN, f32::max) - c.iter().copied().fold(f32::MAX, f32::min)
}

/// SetLum(C, l) of W3C Compositing 1, clipped into gamut.
fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color(c.map(|v| v + d))
}

/// ClipColor(C) of W3C Compositing 1.
fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c.iter().copied().fold(f32::MAX, f32::min);
    let x = c.iter().copied().fold(f32::MIN, f32::max);
    let mut out = c;
    if n < 0.0 && l != n {
        out = out.map(|v| l + (v - l) * l / (l - n));
    }
    if x > 1.0 && x != l {
        out = out.map(|v| l + (v - l) * (1.0 - l) / (x - l));
    }
    out
}

/// SetSat(C, s) of W3C Compositing 1.
fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let mut idx = [0_usize, 1, 2];
    idx.sort_by(|&a, &b| c[a].total_cmp(&c[b]));
    let (min_i, mid_i, max_i) = (idx[0], idx[1], idx[2]);
    let mut out = [0.0_f32; 3];
    if c[max_i] > c[min_i] {
        out[mid_i] = (c[mid_i] - c[min_i]) * s / (c[max_i] - c[min_i]);
        out[max_i] = s;
    }
    out
}
