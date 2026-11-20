//! Role: CSS color spaces: `hsl`, `hwb`, `lab`, `lch`, `oklab`, `oklch`.

use kivact_types::{ColorSpace, SpaceColor};

use super::value::{Color, Repr};

/// HSL color: hue in degrees, saturation and lightness in `0.0..=1.0`.
pub fn hsl(hue: impl Into<f64>, saturation: impl Into<f64>, lightness: impl Into<f64>) -> Color {
    let h = (hue.into()).rem_euclid(360.0) / 60.0;
    let s = (saturation.into()).clamp(0.0, 1.0);
    let l = (lightness.into()).clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    Color::from_rgba_f64(r + m, g + m, b + m, 1.0)
}

/// Oklab color: lightness `0..=1`, and the `a` and `b` axes (about ±0.4).
pub fn oklab(l: impl Into<f64>, a: impl Into<f64>, b: impl Into<f64>) -> Color {
    Color::from_space(ColorSpace::Oklab, l, a, b)
}

/// Oklch color: lightness `0..=1`, chroma (from 0), hue in degrees.
pub fn oklch(l: impl Into<f64>, c: impl Into<f64>, h: impl Into<f64>) -> Color {
    Color::from_space(ColorSpace::Oklch, l, c, h)
}

/// CIE Lab color: lightness `0..=100`, and the `a` and `b` axes.
pub fn lab(l: impl Into<f64>, a: impl Into<f64>, b: impl Into<f64>) -> Color {
    Color::from_space(ColorSpace::Lab, l, a, b)
}

/// CIE LCh color: lightness `0..=100`, chroma (from 0), hue in degrees.
pub fn lch(l: impl Into<f64>, c: impl Into<f64>, h: impl Into<f64>) -> Color {
    Color::from_space(ColorSpace::Lch, l, c, h)
}

/// HWB color: hue in degrees, whiteness and blackness in `0.0..=1.0`.
pub fn hwb(h: impl Into<f64>, w: impl Into<f64>, b: impl Into<f64>) -> Color {
    Color::from_space(ColorSpace::Hwb, h, w, b)
}

impl Color {
    /// A color with three channels in `space`, in the order the space names
    /// them (CSS `lab(l a b)`, `hsl(h s l)`, ..), with full alpha. The
    /// channel ranges are those of the space (Oklab L `0..=1`, Lab L
    /// `0..=100`, Hsl S and L `0..=1`).
    pub fn from_space(
        space: ColorSpace,
        c0: impl Into<f64>,
        c1: impl Into<f64>,
        c2: impl Into<f64>,
    ) -> Color {
        Color(Repr::Space(SpaceColor::new(
            space,
            [c0.into(), c1.into(), c2.into()],
            1.0,
        )))
    }
}
