//! Role: The pop kit's own colors as values a style or a draw call accepts.

use crate::tokens::Tokens;
use kivact::style::{Theme, ThemeColor};

/// A color of the Pop kit's [`Tokens`], read from the theme it is painted in.
///
/// Every color input of a style, a gradient or a draw call takes it, so
/// the color follows a theme switch without a closure that reads the tokens:
/// `style().background(ColorName::Background)`. A theme without the kit's tokens
/// reads the values the tokens fall back to. Take a [`Tokens`] field by hand only to
/// compute a new color from it (`with_alpha`, `mix_in`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorName {
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

impl ThemeColor for ColorName {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.ext::<Tokens>();
        match self {
            ColorName::Background => p.background,
            ColorName::Surface => p.surface,
            ColorName::SurfaceMuted => p.surface_muted,
            ColorName::SurfaceActive => p.surface_active,
            ColorName::Line => p.line,
            ColorName::LineStrong => p.line_strong,
            ColorName::Text => p.text,
            ColorName::TextMuted => p.text_muted,
            ColorName::TextFaint => p.text_faint,
            ColorName::Blue => p.blue,
            ColorName::BlueDeep => p.blue_deep,
            ColorName::Cyan => p.cyan,
            ColorName::Yellow => p.yellow,
            ColorName::Pink => p.pink,
            ColorName::Mint => p.mint,
            ColorName::Red => p.red,
            ColorName::Orange => p.orange,
            ColorName::Purple => p.purple,
            ColorName::Navy => p.navy,
        }
    }
}
