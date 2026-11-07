//! Role: Colors: sRGB, theme tokens, theme-resolved values and colors in a
//! CSS color space, resolved at paint.

mod css;

use kivact_types::{ColorSpace, Rgba, SpaceColor};

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
    /// A color in a CSS color space, as authored (its alpha included); it
    /// is gamut mapped only when it is painted in sRGB.
    Space(SpaceColor),
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

    /// Parse a CSS color (§11.4.1): names, hex, `rgb()`, `hsl()`, `hwb()`,
    /// `lab()`, `lch()`, `oklab()`, `oklch()` and `color()`. Errors are
    /// `ErrorKind::Message` naming the problem.
    #[track_caller]
    pub fn css(text: &str) -> Result<Color, kivact_runtime::UiError> {
        css::parse(text).map_err(kivact_runtime::UiError::msg)
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
