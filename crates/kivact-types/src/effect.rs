//! Role: How a layer's pixels blend with what is beneath (CSS mix-blend-mode).

/// How a layer's pixels combine with what is beneath them (CSS
/// `mix-blend-mode`; [`BlendMode::PlusLighter`] is canvas `lighter`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlendMode {
    /// Source over.
    #[default]
    Normal,
    /// Multiply.
    Multiply,
    /// Screen.
    Screen,
    /// Overlay.
    Overlay,
    /// Darken.
    Darken,
    /// Lighten.
    Lighten,
    /// Color dodge.
    ColorDodge,
    /// Color burn.
    ColorBurn,
    /// Hard light.
    HardLight,
    /// Soft light.
    SoftLight,
    /// Difference.
    Difference,
    /// Exclusion.
    Exclusion,
    /// Hue.
    Hue,
    /// Saturation.
    Saturation,
    /// Color.
    Color,
    /// Luminosity.
    Luminosity,
    /// Additive (CSS `plus-lighter`, canvas `lighter`).
    PlusLighter,
}
