//! Role: Colors: sRGB, theme tokens or theme-resolved values, resolved at paint.

use kivact_types::Rgba;

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
pub(super) enum Repr {
    Rgba(f64, f64, f64, f64),
    Token(ColorToken, f64),
    /// A `ThemeColor` value (see `theme::value`), by its index in the table
    /// of the values stored so far, with the alpha applied to what it
    /// resolves to.
    Themed(u32, f64),
}

/// A concrete color, a theme token or a theme-resolved value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub(super) Repr);

/// Opaque color from `0xRRGGBB`.
pub fn rgb(hex: u32) -> Color {
    rgba(hex, 1.0)
}

/// Color from `0xRRGGBB` and alpha `0.0..=1.0`.
pub fn rgba(hex: u32, alpha: f64) -> Color {
    let c = Rgba::from_hex(hex, alpha);
    Color(Repr::Rgba(c.r, c.g, c.b, c.a))
}

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
        }
    }
}

impl From<ColorToken> for Color {
    fn from(t: ColorToken) -> Self {
        Color::token(t)
    }
}

kivact_runtime::constant_value!(Color);
