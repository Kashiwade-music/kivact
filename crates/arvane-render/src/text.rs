//! Role: How glyphs are rasterized, the same on the CPU and the GPU.
//!
//! Text is antialiased in grayscale (no subpixel ClearType: it breaks on
//! transparent layers and transforms). Coverage is blended in sRGB space,
//! linearly in the encoded values, so a pixel half covered by white text on
//! black becomes sRGB 128, about 22 % luminance: light text loses weight on a
//! dark background, while dark text on a light one keeps it. Three things
//! make small text as crisp and dark as the platform's:
//! - Hinting: both rasterizers hint vertically only (skrifa's light
//!   "smooth" target, the font's instructions or the autohinter) at the
//!   device font size, and put each run's baseline on a whole device
//!   pixel. Horizontal positions keep their subpixel precision, so the
//!   shaped advances and the layout are unchanged. This needs a
//!   translation and a uniform scale (`Transform::uniform_scale`, within a
//!   relative 1e-5, so a skew and the counter-skew of a child count as the
//!   translation they leave); under a rotation, skew or non-uniform scale
//!   glyphs are drawn unhinted (`draw.rs`).
//! - Stem darkening, which depends on the device font size and on the
//!   luminance of the text color. Each side of a glyph's outline is dilated
//!   by `d(size, L) = D_FULL * min(1, L_q / L_SAT) * F(size)` device pixels
//!   ([`darkening`]). Light text gets up to [`D_FULL`] and dark text none:
//!   the dilation rises linearly with the luminance until [`L_SAT`], where
//!   it saturates. `F(size)` is 1 up to [`S_FULL`] device pixels, fades
//!   linearly to 0 at [`S_NONE`], and is 0 above.
//! - Positions: each glyph's horizontal origin is put on a quarter device
//!   pixel ([`subpixel`]). The GPU draws glyphs from an atlas that keeps
//!   four subpixel positions per glyph (glifo's cache), copying them
//!   pixel-aligned; the CPU draws the outlines at the same positions, so
//!   both produce the same coverage. A quarter pixel moves a glyph by at
//!   most an eighth of a pixel from its shaped position.
//!
//! The luminance `L` is the relative luminance of the text color (sRGB
//! decoded per channel, Rec. 709 weights, alpha ignored), quantized to
//! eight buckets, `L_q = round(7 L) / 7` ([`luminance_bucket`]). The
//! quantization bounds the cached glyphs: glifo's glyph atlas key includes
//! the embolden amount, so each bucket is a separate cached glyph on the CPU
//! and on the GPU, at most eight per glyph, size and subpixel position
//! instead of one per color. A non-solid paint (gradient, image, pattern)
//! has no single luminance and uses the middle bucket, `4/7`
//! ([`run_darkening`]). Outline strokes are never darkened.
//!
//! Why the darkening goes into the outlines and not into the coverage:
//! neither vello_cpu nor vello_hybrid offers a gamma or contrast adjustment
//! of the coverage after rasterization, and on the GPU the glyph atlas pages
//! are drawn by the GPU itself, so the CPU never sees their pixels.
//! Dilating the outline before rasterization is the one lever both backends
//! take in the same way.
//!
//! Calibration: the constants were fitted to Chrome's grayscale rendering on
//! Windows (`--disable-lcd-text`, which is DirectWrite grayscale), with Kanit,
//! Kanit Mono and Inter at 10 to 48 px in seven color pairs. Dark text needs
//! no darkening, because the grayscale sRGB blending already makes it as
//! heavy as DirectWrite; light text needs about 0.21 px per side.

use kivact_types::color::Rgba;
use vello_cpu::kurbo::Diagonal2;

/// Dilation per side, in device pixels, of a glyph whose text color has a
/// luminance of [`L_SAT`] or more, at a size up to [`S_FULL`].
const D_FULL: f64 = 0.21;
/// The luminance from which the full dilation [`D_FULL`] applies.
const L_SAT: f64 = 0.8;
/// The device font size, in pixels, up to which the full dilation applies.
const S_FULL: f64 = 48.0;
/// The device font size, in pixels, at and above which no dilation applies.
const S_NONE: f64 = 96.0;

/// Luminance buckets: `L_q = round(L * BUCKET_STEPS) / BUCKET_STEPS`, eight
/// values `k / 7` for `k` in `0..=7`.
const BUCKET_STEPS: f64 = 7.0;
/// The bucket of a non-solid paint: `4 / 7`, the middle of the eight.
const NON_SOLID_BUCKET: f64 = 4.0 / 7.0;

/// Stem darkening of a glyph at a device font size of `size` pixels, in a
/// text color of luminance bucket `bucket` (`L_q`): the dilation per side,
/// `d(size, L)`, with the calibrated constants.
pub(crate) fn darkening(size: f64, bucket: f64) -> Diagonal2 {
    let amount = dilation(size, bucket, D_FULL, L_SAT, S_FULL, S_NONE);
    Diagonal2::new(amount, amount)
}

/// The stem darkening of a run drawn at a device font size of `size` pixels.
/// `solid` is the color of its solid paint; `None` is a non-solid paint,
/// which takes the middle bucket.
pub(crate) fn run_darkening(size: f64, solid: Option<Rgba>) -> Diagonal2 {
    darkening(size, solid.map_or(NON_SOLID_BUCKET, luminance_bucket))
}

/// `d(size, L) = D_FULL * min(1, L_q / L_SAT) * F(size)` for the given
/// constants: `bucket` is `L_q`, and `F` is 1 up to `s_full`, linear to 0 at
/// `s_none`, and 0 above.
fn dilation(size: f64, bucket: f64, d_full: f64, l_sat: f64, s_full: f64, s_none: f64) -> f64 {
    let per_luminance = d_full * (bucket / l_sat).min(1.0);
    let fade = ((s_none - size) / (s_none - s_full)).clamp(0.0, 1.0);
    per_luminance * fade
}

/// The quantized luminance bucket `L_q = round(7 L) / 7` of a solid color.
pub(crate) fn luminance_bucket(color: Rgba) -> f64 {
    (luminance(color) * BUCKET_STEPS).round() / BUCKET_STEPS
}

/// The relative luminance of `color`: each sRGB channel decoded to linear
/// light, weighted by the Rec. 709 coefficients. Alpha is ignored.
fn luminance(color: Rgba) -> f64 {
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

/// One sRGB channel in `0..=1`, decoded to linear light.
fn linear(channel: f64) -> f64 {
    let c = channel.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// `x` (logical pixels, at `scale` device pixels per logical pixel) on the
/// nearest quarter device pixel.
pub(crate) fn subpixel(x: f64, scale: f64) -> f64 {
    (x * scale * 4.0).round() / (scale * 4.0)
}

#[cfg(test)]
mod tests {
    //! Role: Unit tests of the stem darkening model and of the luminance buckets.

    use super::*;

    const BLACK: Rgba = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    const WHITE: Rgba = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// The shipped model, with its constants passed in.
    fn model(size: f64, bucket: f64) -> f64 {
        dilation(size, bucket, D_FULL, L_SAT, S_FULL, S_NONE)
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn dilation_is_non_decreasing_in_luminance() {
        let values: Vec<f64> = (0..=7).map(|k| model(12.0, f64::from(k) / 7.0)).collect();
        for pair in values.windows(2) {
            assert!(pair[0] <= pair[1], "{values:?}");
        }
    }

    #[test]
    fn black_text_gets_no_darkening() {
        assert!(near(model(12.0, 0.0), 0.0));
        assert_eq!(darkening(12.0, 0.0).xx, 0.0);
    }

    #[test]
    fn light_text_at_saturation_gets_the_full_dilation_up_to_s_full() {
        // Buckets 6/7 and 1 are at or above L_SAT = 0.8.
        for bucket in [6.0 / 7.0, 1.0] {
            assert!(near(model(1.0, bucket), D_FULL));
            assert!(near(model(24.0, bucket), D_FULL));
            assert!(near(model(48.0, bucket), D_FULL));
        }
        // Bucket 5/7 is below it, so it gets less.
        assert!(model(24.0, 5.0 / 7.0) < D_FULL);
    }

    #[test]
    fn dilation_fades_linearly_to_zero_at_s_none() {
        assert!(near(model(72.0, 1.0), D_FULL / 2.0));
        assert!(near(model(96.0, 1.0), 0.0));
        assert!(near(model(120.0, 1.0), 0.0));
    }

    #[test]
    fn dilation_does_not_grow_with_size() {
        let values: Vec<f64> = (0..=120).map(|s| model(f64::from(s), 1.0)).collect();
        for pair in values.windows(2) {
            assert!(pair[1] <= pair[0], "{values:?}");
        }
    }

    #[test]
    fn luminance_buckets_are_eight_values_k_over_7() {
        let mut seen = [false; 8];
        for step in 0..=1000 {
            let v = f64::from(step) / 1000.0;
            let bucket = luminance_bucket(Rgba {
                r: v,
                g: v,
                b: v,
                a: 1.0,
            });
            let k = bucket * 7.0;
            assert!((k - k.round()).abs() < 1e-12, "{bucket}");
            seen[k.round() as usize] = true;
        }
        assert!(seen.iter().all(|s| *s), "{seen:?}");
    }

    #[test]
    fn white_is_the_brightest_bucket_and_black_the_darkest() {
        assert_eq!(luminance_bucket(WHITE), 1.0);
        assert_eq!(luminance_bucket(BLACK), 0.0);
    }

    #[test]
    fn alpha_does_not_change_the_bucket() {
        let transparent_white = Rgba { a: 0.0, ..WHITE };
        assert_eq!(luminance_bucket(transparent_white), 1.0);
    }

    #[test]
    fn non_solid_paint_uses_the_middle_bucket() {
        assert_eq!(NON_SOLID_BUCKET, 4.0 / 7.0);
        for size in [8.0, 24.0, 36.0, 60.0] {
            assert_eq!(run_darkening(size, None), darkening(size, 4.0 / 7.0));
        }
    }
}
