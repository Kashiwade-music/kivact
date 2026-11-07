// Source: resvg 0.46.0, src/filter/composite.rs (Copyright 2020 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Changes: the arithmetic operator takes f64 coefficients and writes through
// the compositor's premultiplied pixmap.

//! Role: feComposite with the arithmetic operator on premultiplied colors.

use crate::cpu::surface::Pixmap;

/// The coefficients k1..k4 of the arithmetic operator.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Coefficients {
    /// The product coefficient.
    pub(crate) k1: f32,
    /// The first image coefficient.
    pub(crate) k2: f32,
    /// The second image coefficient.
    pub(crate) k3: f32,
    /// The constant.
    pub(crate) k4: f32,
}

/// `result = k1·i1·i2 + k2·i1 + k3·i2 + k4` per channel of the premultiplied `first` and `second`.
/// The color is bounded by the result's alpha; a zero alpha gives transparent black.
pub(crate) fn arithmetic(k: Coefficients, first: &Pixmap, second: &Pixmap) -> Pixmap {
    let (w, h) = first.size();
    let mut out = Pixmap::new(w, h);
    let calc = |i1: f32, i2: f32, max: f32| {
        (k.k1 * i1 * i2 + k.k2 * i1 + k.k3 * i2 + k.k4).clamp(0.0, max)
    };
    for y in 0..h {
        for x in 0..w {
            let c1 = first.get(x, y);
            let c2 = second.get(x, y);
            let a = calc(c1[3], c2[3], 1.0);
            if a <= f32::EPSILON {
                continue;
            }
            let color = [0, 1, 2].map(|i| calc(c1[i], c2[i], a));
            out.set(x, y, [color[0], color[1], color[2], a]);
        }
    }
    out
}
