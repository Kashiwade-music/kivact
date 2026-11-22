//! Role: `Color` and its representation: sRGB, theme variables, CSS color spaces.

use kivact_types::{Rgba, SpaceColor, ThemeVar};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Repr {
    Rgba(f64, f64, f64, f64),
    /// A theme variable, with the alpha applied to what it resolves to.
    Var(&'static ThemeVar<Color>, f64),
    /// A color in a CSS color space, as authored (its alpha included); it
    /// is gamut mapped only when it is painted in sRGB.
    Space(SpaceColor),
}

/// A concrete color, a theme variable or a color in a CSS color space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub(crate) Repr);

/// Opaque color from `0xRRGGBB`.
pub const fn rgb(hex: u32) -> Color {
    rgba(hex, 1.0)
}

/// Color from `0xRRGGBB` and alpha `0.0..=1.0`.
pub const fn rgba(hex: u32, alpha: f64) -> Color {
    let c = Rgba::from_hex(hex, alpha);
    Color(Repr::Rgba(c.r, c.g, c.b, c.a))
}

impl Color {
    pub const TRANSPARENT: Color = Color(Repr::Rgba(0.0, 0.0, 0.0, 0.0));
    pub const BLACK: Color = Color(Repr::Rgba(0.0, 0.0, 0.0, 1.0));
    pub const WHITE: Color = Color(Repr::Rgba(1.0, 1.0, 1.0, 1.0));

    /// A theme variable as a color: what the theme sets it to, resolved
    /// where the color is used (its alpha is multiplied by
    /// [`with_alpha`](Self::with_alpha)).
    pub const fn var(v: &'static ThemeVar<Color>) -> Color {
        Color(Repr::Var(v, 1.0))
    }

    /// The variable and alpha of a color that is a theme variable.
    pub(crate) fn var_target(self) -> Option<(&'static ThemeVar<Color>, f64)> {
        match self.0 {
            Repr::Var(v, a) => Some((v, a)),
            _ => None,
        }
    }

    /// Components in `0.0..=1.0`.
    pub fn from_rgba_f64(r: f64, g: f64, b: f64, a: f64) -> Color {
        Color(Repr::Rgba(r, g, b, a.clamp(0.0, 1.0)))
    }

    /// Multiply alpha.
    pub fn with_alpha(self, alpha: impl Into<f64>) -> Color {
        let alpha = alpha.into();
        match self.0 {
            Repr::Rgba(r, g, b, a) => Color(Repr::Rgba(r, g, b, (a * alpha).clamp(0.0, 1.0))),
            Repr::Var(v, a) => Color(Repr::Var(v, (a * alpha).clamp(0.0, 1.0))),
            Repr::Space(mut c) => {
                c.alpha = (c.alpha * alpha).clamp(0.0, 1.0);
                Color(Repr::Space(c))
            }
        }
    }
}

kivact_runtime::constant_value!(Color);
