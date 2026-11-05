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
