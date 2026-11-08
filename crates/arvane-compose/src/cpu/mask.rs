//! Role: Clip coverage and mask layers applied to a surface.

use kivact_types::paint::{MaskComposite, MaskMode};

use super::surface::{Pixmap, Premul, Texels};

/// Multiplies every pixel of `target` by the alpha of `coverage` at the same pixel.
pub(crate) fn clip(target: &mut Pixmap, coverage: &Pixmap) {
    let (w, h) = target.size();
    for y in 0..h {
        for x in 0..w {
            let k = coverage.texel(i64::from(x), i64::from(y))[3];
            let c = target.get(x, y);
            target.set(x, y, c.map(|v| v * k));
        }
    }
}

/// Multiplies every pixel of `target` by the combined mask of `layers`, given with the bottom layer last.
pub(crate) fn masks(target: &mut Pixmap, layers: &[(&Pixmap, MaskMode, MaskComposite)]) {
    let (w, h) = target.size();
    for y in 0..h {
        for x in 0..w {
            let acc = combined(layers, x, y);
            let c = target.get(x, y);
            target.set(x, y, c.map(|v| v * acc));
        }
    }
}

/// The combined mask value at pixel (`x`, `y`): the bottom layer's value, then each layer above combined by its composite.
fn combined(layers: &[(&Pixmap, MaskMode, MaskComposite)], x: u32, y: u32) -> f32 {
    let mut iter = layers.iter().rev();
    let Some((bottom, mode, _)) = iter.next() else {
        return 1.0;
    };
    let mut acc = value(bottom, *mode, x, y);
    for (layer, mode, composite) in iter {
        let m = value(layer, *mode, x, y);
        acc = match composite {
            MaskComposite::Add => m + acc * (1.0 - m),
            MaskComposite::Subtract => m * (1.0 - acc),
            MaskComposite::Intersect => m * acc,
            MaskComposite::Exclude => m * (1.0 - acc) + acc * (1.0 - m),
        };
    }
    acc
}

/// One mask layer's value at (`x`, `y`): its alpha, or its luminance times its alpha.
fn value(mask: &Pixmap, mode: MaskMode, x: u32, y: u32) -> f32 {
    let c: Premul = mask.texel(i64::from(x), i64::from(y));
    match mode {
        MaskMode::Alpha => c[3],
        MaskMode::Luminance => {
            let a = c[3];
            if a <= 0.0 {
                return 0.0;
            }
            (0.2125 * c[0] / a + 0.7154 * c[1] / a + 0.0721 * c[2] / a) * a
        }
    }
}
