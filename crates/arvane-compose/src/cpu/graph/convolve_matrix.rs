// Source: resvg 0.46.0, src/filter/convolve_matrix.rs (Copyright 2020 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Changes: the kernel, divisor, bias and edge mode are kivact's ConvolveMatrix
// fields; a zero divisor becomes the kernel sum, or 1 when the sum is 0.

//! Role: feConvolveMatrix on the CPU.

use kivact_types::effect::EdgeMode;

use crate::cpu::surface::Pixmap;

/// The parameters of one convolution.
pub(crate) struct Convolution<'a> {
    /// Columns and rows of the kernel.
    pub(crate) order: (u32, u32),
    /// The kernel, row by row.
    pub(crate) kernel: &'a [f64],
    /// The divisor (0 means the kernel sum, or 1 when that is 0).
    pub(crate) divisor: f64,
    /// The bias.
    pub(crate) bias: f64,
    /// The target pixel of the kernel.
    pub(crate) target: (u32, u32),
    /// How pixels beyond the edge are read.
    pub(crate) edge: EdgeMode,
    /// Whether the alpha is kept (the input is then unpremultiplied by the caller).
    pub(crate) preserve_alpha: bool,
}

/// Convolves `src` in place.
pub(crate) fn apply(conv: &Convolution<'_>, src: &mut Pixmap) {
    let (cols, rows) = (conv.order.0 as i64, conv.order.1 as i64);
    let divisor = if conv.divisor == 0.0 {
        let sum: f64 = conv.kernel.iter().sum();
        if sum == 0.0 { 1.0 } else { sum }
    } else {
        conv.divisor
    } as f32;
    let bias = conv.bias as f32;
    let (w, h) = src.size();
    let (wi, hi) = (i64::from(w), i64::from(h));
    let original = src.data.clone();
    let read = |x: i64, y: i64| -> Option<[f32; 4]> {
        if !((0..wi).contains(&x) && (0..hi).contains(&y)) {
            return None;
        }
        let i = (y as usize * w as usize + x as usize) * 4;
        let p = &original[i..i + 4];
        Some(std::array::from_fn(|c| f32::from(p[c]) / 255.0))
    };
    for y in 0..hi {
        for x in 0..wi {
            let mut acc = [0.0_f32; 4];
            for oy in 0..rows {
                for ox in 0..cols {
                    let mut tx = x - i64::from(conv.target.0) + ox;
                    let mut ty = y - i64::from(conv.target.1) + oy;
                    match conv.edge {
                        EdgeMode::None => {
                            if tx < 0 || tx >= wi || ty < 0 || ty >= hi {
                                continue;
                            }
                        }
                        EdgeMode::Duplicate => {
                            tx = tx.clamp(0, wi - 1);
                            ty = ty.clamp(0, hi - 1);
                        }
                        EdgeMode::Wrap => {
                            tx = tx.rem_euclid(wi);
                            ty = ty.rem_euclid(hi);
                        }
                    }
                    let k = conv.kernel[((rows - oy - 1) * cols + (cols - ox - 1)) as usize] as f32;
                    let Some(p) = read(tx, ty) else {
                        continue;
                    };
                    acc[0] += p[0] * k;
                    acc[1] += p[1] * k;
                    acc[2] += p[2] * k;
                    if !conv.preserve_alpha {
                        acc[3] += p[3] * k;
                    }
                }
            }
            let i = (y as usize * w as usize + x as usize) * 4;
            let in_alpha = f32::from(original[i + 3]) / 255.0;
            let new_a = if conv.preserve_alpha {
                in_alpha
            } else {
                acc[3] / divisor + bias
            };
            let bounded_a = new_a.clamp(0.0, 1.0);
            let channel = |v: f32| {
                let v = v / divisor + bias * new_a;
                if conv.preserve_alpha {
                    v.clamp(0.0, 1.0) * bounded_a
                } else {
                    v.clamp(0.0, bounded_a)
                }
            };
            let out = [channel(acc[0]), channel(acc[1]), channel(acc[2]), bounded_a];
            src.data[i..i + 4].copy_from_slice(&out.map(|v| (v * 255.0 + 0.5) as u8));
        }
    }
}
