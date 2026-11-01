//! Role: The pop kit's own colors as values a style or a draw call accepts.

use crate::tokens::Tokens;
use kivact::style::{Theme, ThemeColor};

/// A color of the Pop kit's [`Tokens`], read from the theme it is painted in.
///
/// Every color input of a style, a gradient or a draw call takes it, so
/// the color follows a theme switch without a closure that reads the tokens:
/// `style().background(Color::Background)`. A theme without the kit's tokens
/// reads the values the tokens fall back to. Take a [`Tokens`] field by hand only to
/// compute a new color from it (`with_alpha`, `mix_in`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// [`Tokens::background`].
    Background,
    /// [`Tokens::surface`].
    Surface,
    /// [`Tokens::surface_muted`].
    SurfaceMuted,
    /// [`Tokens::surface_active`].
    SurfaceActive,
    /// [`Tokens::line`].
    Line,
    /// [`Tokens::line_strong`].
    LineStrong,
    /// [`Tokens::text`].
    Text,
    /// [`Tokens::text_muted`].
    TextMuted,
    /// [`Tokens::text_faint`].
    TextFaint,
    /// [`Tokens::blue`].
    Blue,
    /// [`Tokens::blue_deep`].
    BlueDeep,
    /// [`Tokens::cyan`].
    Cyan,
    /// [`Tokens::yellow`].
    Yellow,
    /// [`Tokens::pink`].
    Pink,
    /// [`Tokens::mint`].
    Mint,
    /// [`Tokens::red`].
    Red,
    /// [`Tokens::orange`].
    Orange,
    /// [`Tokens::purple`].
    Purple,
    /// [`Tokens::navy`].
    Navy,
}

impl ThemeColor for Color {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.get::<Tokens>();
        match self {
            Color::Background => p.background,
            Color::Surface => p.surface,
            Color::SurfaceMuted => p.surface_muted,
            Color::SurfaceActive => p.surface_active,
            Color::Line => p.line,
            Color::LineStrong => p.line_strong,
            Color::Text => p.text,
            Color::TextMuted => p.text_muted,
            Color::TextFaint => p.text_faint,
            Color::Blue => p.blue,
            Color::BlueDeep => p.blue_deep,
            Color::Cyan => p.cyan,
            Color::Yellow => p.yellow,
            Color::Pink => p.pink,
            Color::Mint => p.mint,
            Color::Red => p.red,
            Color::Orange => p.orange,
            Color::Purple => p.purple,
            Color::Navy => p.navy,
        }
    }
}
