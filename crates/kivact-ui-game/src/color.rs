//! Role: The game kit's own colors as values a style or a draw call accepts.

use crate::tokens::Tokens;
use kivact::style::{Theme, ThemeColor};

/// A color of the Game kit's [`Tokens`], read from the theme it is painted in.
///
/// Every color input of a style, a gradient or a draw call takes it, so
/// the color follows a theme switch without a closure that reads the tokens:
/// `style().background(Color::PrimaryHover)`. A theme without the kit's tokens
/// reads the values the tokens fall back to. Take a [`Tokens`] field by hand only to
/// compute a new color from it (`with_alpha`, `mix_in`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
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

impl ThemeColor for Color {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.get::<Tokens>();
        match self {
            Color::PrimaryHover => p.primary_hover,
            Color::PrimaryStrong => p.primary_strong,
            Color::PrimaryWeak => p.primary_weak,
            Color::Cyan => p.cyan,
            Color::Yellow => p.yellow,
            Color::Navy => p.navy,
            Color::Canvas => p.canvas,
            Color::Surface3 => p.surface_3,
            Color::LineStrong => p.line_strong,
            Color::Text2 => p.text_2,
            Color::Text3 => p.text_3,
            Color::Sidebar => p.sidebar,
            Color::SidebarAlt => p.sidebar_alt,
            Color::SidebarText => p.sidebar_text,
            Color::SidebarDim => p.sidebar_dim,
        }
    }
}
