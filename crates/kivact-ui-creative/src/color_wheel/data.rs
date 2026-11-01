//! Role: Public color field model value.
/// The channels displayed below a color wheel. Model selection is UI state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ColorModel {
    /// Normalized red, green and blue channels.
    #[default]
    Rgb,
    /// Hue in degrees and normalized saturation and value.
    Hsv,
    /// A hexadecimal RGB or RGBA draft.
    Hex,
}
kivact::view::constant_value!(ColorModel);
