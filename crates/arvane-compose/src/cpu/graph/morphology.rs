// Source: resvg 0.46.0, src/filter/morphology.rs (Copyright 2020 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Changes: radii are given in pixels; the pixel type is the compositor's
// premultiplied RGBA8 pixmap.

//! Role: feMorphology on the CPU (erode and dilate over a pixel window).

use kivact_types::effect::MorphologyOp;

use crate::cpu::surface::Pixmap;

/// Erodes or dilates `src` in place with the window of `rx` × `ry` px (each side rounded up).
pub(crate) fn apply(operator: MorphologyOp, rx: f64, ry: f64, src: &mut Pixmap) {
    let (width, height) = src.size();
    let columns = ((rx.ceil() as u32).saturating_mul(2)).min(width);
    let rows = ((ry.ceil() as u32).saturating_mul(2)).min(height);
    let target_x = (columns as f32 / 2.0).floor() as i64;
    let target_y = (rows as f32 / 2.0).floor() as i64;
    let erode = operator == MorphologyOp::Erode;
    let source = src.data.clone();
    let texel = |x: i64, y: i64| -> [u8; 4] {
        let i = (y as usize * width as usize + x as usize) * 4;
        [source[i], source[i + 1], source[i + 2], source[i + 3]]
    };
    for y in 0..i64::from(height) {
        for x in 0..i64::from(width) {
            let mut best = if erode { [255_u8; 4] } else { [0_u8; 4] };
            for oy in 0..i64::from(rows) {
                for ox in 0..i64::from(columns) {
                    let tx = x - target_x + ox;
                    let ty = y - target_y + oy;
                    if tx < 0 || tx >= i64::from(width) || ty < 0 || ty >= i64::from(height) {
                        continue;
                    }
                    let p = texel(tx, ty);
                    for c in 0..4 {
                        best[c] = if erode {
                            best[c].min(p[c])
                        } else {
                            best[c].max(p[c])
                        };
                    }
                }
            }
            let i = (y as usize * width as usize + x as usize) * 4;
            src.data[i..i + 4].copy_from_slice(&best);
        }
    }
}
