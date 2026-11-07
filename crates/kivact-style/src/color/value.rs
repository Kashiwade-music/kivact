//! Role: `Color` and its representation: sRGB, theme tokens and theme values.

use kivact_types::{Rgba, SpaceColor};

/// Semantic color slots of a [`Theme`](super::theme::value::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorToken {
    Surface,
    SurfaceAlt,
    Text,
    TextMuted,
    Accent,
    AccentText,
    Border,
    Focus,
    Danger,
    Success,
    Warning,
    Overlay,
    Shadow,
    Selection,
    /// A tint laid over a surface under the pointer. Often translucent, so
    /// it works over any background.
    Hover,
    /// A tint laid over a surface while it is pressed; stronger than
    /// [`Hover`](Self::Hover).
    Pressed,
    /// The background of a surface that floats above the page: a menu, a
    /// popover or a tooltip.
    Raised,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Repr {
    Rgba(f64, f64, f64, f64),
    Token(ColorToken, f64),
    /// A `ThemeColor` value (see `theme::value`), by its index in the table
    /// of the values stored so far, with the alpha applied to what it
    /// resolves to.
    Themed(u32, f64),
    /// A color in a CSS color space, as authored (its alpha included); it
    /// is gamut mapped only when it is painted in sRGB.
    Space(SpaceColor),
}

/// A concrete color, a theme token or a theme-resolved value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub(crate) Repr);

/// Opaque color from `0xRRGGBB`.
pub fn rgb(hex: u32) -> Color {
    rgba(hex, 1.0)
}

/// Color from `0xRRGGBB` and alpha `0.0..=1.0`.
pub fn rgba(hex: u32, alpha: f64) -> Color {
    let c = Rgba::from_hex(hex, alpha);
    Color(Repr::Rgba(c.r, c.g, c.b, c.a))
}

impl Color {
    pub const TRANSPARENT: Color = Color(Repr::Rgba(0.0, 0.0, 0.0, 0.0));
    pub const BLACK: Color = Color(Repr::Rgba(0.0, 0.0, 0.0, 1.0));
    pub const WHITE: Color = Color(Repr::Rgba(1.0, 1.0, 1.0, 1.0));

    pub fn token(t: ColorToken) -> Color {
        Color(Repr::Token(t, 1.0))
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
            Repr::Token(t, a) => Color(Repr::Token(t, (a * alpha).clamp(0.0, 1.0))),
            Repr::Themed(c, a) => Color(Repr::Themed(c, (a * alpha).clamp(0.0, 1.0))),
            Repr::Space(mut c) => {
                c.alpha = (c.alpha * alpha).clamp(0.0, 1.0);
                Color(Repr::Space(c))
            }
        }
    }
}

impl From<ColorToken> for Color {
    fn from(t: ColorToken) -> Self {
        Color::token(t)
    }
}

kivact_runtime::constant_value!(Color);
