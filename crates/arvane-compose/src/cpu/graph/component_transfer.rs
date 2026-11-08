// Source: resvg 0.46.0, src/filter/component_transfer.rs (Copyright 2020 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Changes: the functions are kivact's TransferFn; the values are f32 in 0..=1
// and written through the compositor's premultiplied pixmap.

//! Role: feComponentTransfer on the CPU (per-channel transfer functions).

use kivact_types::effect::TransferFn;

use crate::cpu::surface::Pixmap;

/// The four transfer functions of one primitive, for red, green, blue and alpha.
pub(crate) struct Transfers<'a> {
    /// Red.
    pub(crate) r: &'a TransferFn,
    /// Green.
    pub(crate) g: &'a TransferFn,
    /// Blue.
    pub(crate) b: &'a TransferFn,
    /// Alpha.
    pub(crate) a: &'a TransferFn,
}

/// Applies the transfer functions to `src` in place (colors are unpremultiplied for the transfer).
pub(crate) fn apply(funcs: &Transfers<'_>, src: &mut Pixmap) {
    let (w, h) = src.size();
    for y in 0..h {
        for x in 0..w {
            let c = src.get(x, y);
            let a = c[3];
            let straight = if a > 0.0 {
                [c[0] / a, c[1] / a, c[2] / a]
            } else {
                [0.0; 3]
            };
            let rgb = [
                transfer_unless_dummy(funcs.r, straight[0]),
                transfer_unless_dummy(funcs.g, straight[1]),
                transfer_unless_dummy(funcs.b, straight[2]),
            ];
            let alpha = transfer_unless_dummy(funcs.a, a);
            src.set(
                x,
                y,
                [rgb[0] * alpha, rgb[1] * alpha, rgb[2] * alpha, alpha],
            );
        }
    }
}

/// `value` through `func`, or unchanged when the function is the identity or has no table.
fn transfer_unless_dummy(func: &TransferFn, value: f32) -> f32 {
    match func {
        TransferFn::Identity => value,
        TransferFn::Table(values) | TransferFn::Discrete(values) if values.is_empty() => value,
        _ => transfer(func, value).clamp(0.0, 1.0),
    }
}

/// The transfer function `func` of the value `c` in 0..=1.
fn transfer(func: &TransferFn, c: f32) -> f32 {
    match func {
        TransferFn::Identity => c,
        TransferFn::Table(values) => {
            let n = values.len() - 1;
            let k = ((c * n as f32).floor().max(0.0) as usize).min(n);
            if k == n {
                values[k] as f32
            } else {
                let vk = values[k] as f32;
                let vk1 = values[k + 1] as f32;
                vk + (c - k as f32 / n as f32) * n as f32 * (vk1 - vk)
            }
        }
        TransferFn::Discrete(values) => {
            let n = values.len();
            let k = ((c * n as f32).floor().max(0.0) as usize).min(n - 1);
            values[k] as f32
        }
        TransferFn::Linear { slope, intercept } => *slope as f32 * c + *intercept as f32,
        TransferFn::Gamma {
            amplitude,
            exponent,
            offset,
        } => *amplitude as f32 * c.powf(*exponent as f32) + *offset as f32,
    }
}
