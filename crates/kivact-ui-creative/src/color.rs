//! Role: The creative kit's own colors as values a style or a draw call accepts.

use crate::tokens::Tokens;
use kivact::style::{Theme, ThemeColor};

/// A color of the Creative kit's [`Tokens`], read from the theme it is painted in.
///
/// Every color input of a style, a gradient or a draw call takes it, so
/// the color follows a theme switch without a closure that reads the tokens:
/// `style().background(Color::Canvas)`. A theme without the kit's tokens
/// reads the values the tokens fall back to. Take a [`Tokens`] field by hand only to
/// compute a new color from it (`with_alpha`, `mix_in`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
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

impl ThemeColor for Color {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.ext::<Tokens>();
        match self {
            Color::Canvas => p.canvas,
            Color::Island => p.island,
            Color::Header => p.header,
            Color::Strong => p.strong,
            Color::Field => p.field,
            Color::Hair => p.hair,
            Color::HairStrong => p.hair_strong,
            Color::Text => p.text,
            Color::TextHigh => p.text_high,
            Color::Muted => p.muted,
            Color::Dim => p.dim,
            Color::Accent => p.accent,
            Color::AccentHigh => p.accent_high,
            Color::AccentInk => p.accent_ink,
            Color::Selection => p.selection,
            Color::Key => p.key,
            Color::X => p.x,
            Color::Y => p.y,
            Color::Z => p.z,
            Color::Success => p.success,
            Color::Warning => p.warning,
        }
    }
}
