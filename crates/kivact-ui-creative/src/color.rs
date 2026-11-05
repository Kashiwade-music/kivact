//! Role: The creative kit's own colors as values a style or a draw call accepts.

use crate::tokens::Tokens;
use kivact::style::{Theme, ThemeColor};

/// A color of the Creative kit's [`Tokens`], read from the theme it is painted in.
///
/// Every color input of a style, a gradient or a draw call takes it, so
/// the color follows a theme switch without a closure that reads the tokens:
/// `style().background(ColorName::Canvas)`. A theme without the kit's tokens
/// reads the values the tokens fall back to. Take a [`Tokens`] field by hand only to
/// compute a new color from it (`with_alpha`, `mix_in`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorName {
    /// [`Tokens::canvas`].
    Canvas,
    /// [`Tokens::island`].
    Island,
    /// [`Tokens::header`].
    Header,
    /// [`Tokens::strong`].
    Strong,
    /// [`Tokens::field`].
    Field,
    /// [`Tokens::hair`].
    Hair,
    /// [`Tokens::hair_strong`].
    HairStrong,
    /// [`Tokens::text`].
    Text,
    /// [`Tokens::text_high`].
    TextHigh,
    /// [`Tokens::muted`].
    Muted,
    /// [`Tokens::dim`].
    Dim,
    /// [`Tokens::accent`].
    Accent,
    /// [`Tokens::accent_high`].
    AccentHigh,
    /// [`Tokens::accent_ink`].
    AccentInk,
    /// [`Tokens::selection`].
    Selection,
    /// [`Tokens::key`].
    Key,
    /// [`Tokens::x`].
    X,
    /// [`Tokens::y`].
    Y,
    /// [`Tokens::z`].
    Z,
    /// [`Tokens::success`].
    Success,
    /// [`Tokens::warning`].
    Warning,
}

impl ThemeColor for ColorName {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.ext::<Tokens>();
        match self {
            ColorName::Canvas => p.canvas,
            ColorName::Island => p.island,
            ColorName::Header => p.header,
            ColorName::Strong => p.strong,
            ColorName::Field => p.field,
            ColorName::Hair => p.hair,
            ColorName::HairStrong => p.hair_strong,
            ColorName::Text => p.text,
            ColorName::TextHigh => p.text_high,
            ColorName::Muted => p.muted,
            ColorName::Dim => p.dim,
            ColorName::Accent => p.accent,
            ColorName::AccentHigh => p.accent_high,
            ColorName::AccentInk => p.accent_ink,
            ColorName::Selection => p.selection,
            ColorName::Key => p.key,
            ColorName::X => p.x,
            ColorName::Y => p.y,
            ColorName::Z => p.z,
            ColorName::Success => p.success,
            ColorName::Warning => p.warning,
        }
    }
}

// Fixed paint colors shared by Creative surfaces and initial domain values.
pub(crate) fn transparent_ink() -> kivact::style::Color {
    kivact::style::rgba(0, 0.0)
}
pub(crate) fn button_surface() -> kivact::style::Color {
    kivact::style::rgba(0xffffff, 0.065)
}
pub(crate) fn control_border() -> kivact::style::Color {
    kivact::style::rgba(0xffffff, 0.04)
}
pub(crate) fn overlay_border() -> kivact::style::Color {
    kivact::style::rgba(0xffffff, 0.1)
}
pub(crate) fn button_pressed() -> kivact::style::Color {
    kivact::style::rgba(0xffffff, 0.05)
}
pub(crate) fn menu_hover() -> kivact::style::Color {
    kivact::style::rgba(0xffffff, 0.07)
}
pub(crate) fn control_shadow() -> kivact::style::Color {
    kivact::style::rgba(0x000000, 0.28)
}
pub(crate) fn overlay_surface() -> kivact::style::Color {
    kivact::style::rgba(0x1e1f26, 0.94)
}
pub(crate) fn window_shadow() -> kivact::style::Color {
    kivact::style::rgba(0x000000, 0.65)
}
pub(crate) fn transparent_backdrop() -> kivact::style::Color {
    kivact::style::rgba(0x050608, 0.0)
}
pub(crate) fn modal_backdrop() -> kivact::style::Color {
    kivact::style::rgba(0x050608, 0.35)
}
pub(crate) fn dialog_shadow() -> kivact::style::Color {
    kivact::style::rgba(0, 0.5)
}
pub(crate) fn menu_shadow() -> kivact::style::Color {
    kivact::style::rgba(0, 0.6)
}
pub(crate) fn default_item_color() -> kivact::style::Color {
    kivact::style::rgb(0x8b7cff)
}
pub(crate) fn driven_property_color() -> kivact::style::Color {
    kivact::style::rgb(0xc084fc)
}
pub(crate) fn checker_color(light: bool) -> kivact::style::Color {
    kivact::style::rgb(if light { 0x57585d } else { 0x272830 })
}
