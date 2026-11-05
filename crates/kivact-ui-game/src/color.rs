//! Role: The game kit's own colors as values a style or a draw call accepts.

use crate::tokens::Tokens;
use kivact::style::{Theme, ThemeColor};

/// A color of the Game kit's [`Tokens`], read from the theme it is painted in.
///
/// Every color input of a style, a gradient or a draw call takes it, so
/// the color follows a theme switch without a closure that reads the tokens:
/// `style().background(ColorName::PrimaryHover)`. A theme without the kit's tokens
/// reads the values the tokens fall back to. Take a [`Tokens`] field by hand only to
/// compute a new color from it (`with_alpha`, `mix_in`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorName {
    /// [`Tokens::primary_hover`].
    PrimaryHover,
    /// [`Tokens::primary_strong`].
    PrimaryStrong,
    /// [`Tokens::primary_weak`].
    PrimaryWeak,
    /// [`Tokens::cyan`].
    Cyan,
    /// [`Tokens::yellow`].
    Yellow,
    /// [`Tokens::navy`].
    Navy,
    /// [`Tokens::canvas`].
    Canvas,
    /// [`Tokens::surface_3`].
    Surface3,
    /// [`Tokens::line_strong`].
    LineStrong,
    /// [`Tokens::text_2`].
    Text2,
    /// [`Tokens::text_3`].
    Text3,
    /// [`Tokens::sidebar`].
    Sidebar,
    /// [`Tokens::sidebar_alt`].
    SidebarAlt,
    /// [`Tokens::sidebar_text`].
    SidebarText,
    /// [`Tokens::sidebar_dim`].
    SidebarDim,
}

impl ThemeColor for ColorName {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.ext::<Tokens>();
        match self {
            ColorName::PrimaryHover => p.primary_hover,
            ColorName::PrimaryStrong => p.primary_strong,
            ColorName::PrimaryWeak => p.primary_weak,
            ColorName::Cyan => p.cyan,
            ColorName::Yellow => p.yellow,
            ColorName::Navy => p.navy,
            ColorName::Canvas => p.canvas,
            ColorName::Surface3 => p.surface_3,
            ColorName::LineStrong => p.line_strong,
            ColorName::Text2 => p.text_2,
            ColorName::Text3 => p.text_3,
            ColorName::Sidebar => p.sidebar,
            ColorName::SidebarAlt => p.sidebar_alt,
            ColorName::SidebarText => p.sidebar_text,
            ColorName::SidebarDim => p.sidebar_dim,
        }
    }
}
