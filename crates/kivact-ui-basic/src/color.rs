//! Role: The basic kit's own colors as values a style or a draw call accepts.

use crate::tokens::Tokens;
use kivact::style::{Theme, ThemeColor};

/// A color of the Basic kit's [`Tokens`], read from the theme it is painted in.
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
    /// [`Tokens::foreground`].
    Foreground,
    /// [`Tokens::card`].
    Card,
    /// [`Tokens::card_foreground`].
    CardForeground,
    /// [`Tokens::popover`].
    Popover,
    /// [`Tokens::popover_foreground`].
    PopoverForeground,
    /// [`Tokens::primary`].
    Primary,
    /// [`Tokens::primary_foreground`].
    PrimaryForeground,
    /// [`Tokens::secondary`].
    Secondary,
    /// [`Tokens::secondary_foreground`].
    SecondaryForeground,
    /// [`Tokens::muted`].
    Muted,
    /// [`Tokens::muted_foreground`].
    MutedForeground,
    /// [`Tokens::accent`].
    Accent,
    /// [`Tokens::accent_foreground`].
    AccentForeground,
    /// [`Tokens::destructive`].
    Destructive,
    /// [`Tokens::border`].
    Border,
    /// [`Tokens::input`].
    Input,
    /// [`Tokens::ring`].
    Ring,
    /// [`Tokens::sidebar`].
    Sidebar,
    /// [`Tokens::sidebar_foreground`].
    SidebarForeground,
    /// [`Tokens::sidebar_primary`].
    SidebarPrimary,
    /// [`Tokens::sidebar_primary_foreground`].
    SidebarPrimaryForeground,
    /// [`Tokens::sidebar_accent`].
    SidebarAccent,
    /// [`Tokens::sidebar_accent_foreground`].
    SidebarAccentForeground,
    /// [`Tokens::sidebar_border`].
    SidebarBorder,
    /// [`Tokens::sidebar_ring`].
    SidebarRing,
    /// [`Tokens::chart`] entry 1.
    Chart1,
    /// [`Tokens::chart`] entry 2.
    Chart2,
    /// [`Tokens::chart`] entry 3.
    Chart3,
    /// [`Tokens::chart`] entry 4.
    Chart4,
    /// [`Tokens::chart`] entry 5.
    Chart5,
    /// [`CodeColors::keyword`](crate::CodeColors::keyword).
    CodeKeyword,
    /// [`CodeColors::string`](crate::CodeColors::string).
    CodeString,
    /// [`CodeColors::number`](crate::CodeColors::number).
    CodeNumber,
    /// [`CodeColors::comment`](crate::CodeColors::comment).
    CodeComment,
    /// [`CodeColors::function`](crate::CodeColors::function).
    CodeFunction,
    /// [`CodeColors::type_name`](crate::CodeColors::type_name).
    CodeTypeName,
    /// [`CodeColors::macro_call`](crate::CodeColors::macro_call).
    CodeMacroCall,
    /// [`CodeColors::lifetime`](crate::CodeColors::lifetime).
    CodeLifetime,
    /// [`CodeColors::attribute`](crate::CodeColors::attribute).
    CodeAttribute,
}

impl ThemeColor for Color {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.get::<Tokens>();
        match self {
            Color::Background => p.background,
            Color::Foreground => p.foreground,
            Color::Card => p.card,
            Color::CardForeground => p.card_foreground,
            Color::Popover => p.popover,
            Color::PopoverForeground => p.popover_foreground,
            Color::Primary => p.primary,
            Color::PrimaryForeground => p.primary_foreground,
            Color::Secondary => p.secondary,
            Color::SecondaryForeground => p.secondary_foreground,
            Color::Muted => p.muted,
            Color::MutedForeground => p.muted_foreground,
            Color::Accent => p.accent,
            Color::AccentForeground => p.accent_foreground,
            Color::Destructive => p.destructive,
            Color::Border => p.border,
            Color::Input => p.input,
            Color::Ring => p.ring,
            Color::Sidebar => p.sidebar,
            Color::SidebarForeground => p.sidebar_foreground,
            Color::SidebarPrimary => p.sidebar_primary,
            Color::SidebarPrimaryForeground => p.sidebar_primary_foreground,
            Color::SidebarAccent => p.sidebar_accent,
            Color::SidebarAccentForeground => p.sidebar_accent_foreground,
            Color::SidebarBorder => p.sidebar_border,
            Color::SidebarRing => p.sidebar_ring,
            Color::Chart1 => p.chart[0],
            Color::Chart2 => p.chart[1],
            Color::Chart3 => p.chart[2],
            Color::Chart4 => p.chart[3],
            Color::Chart5 => p.chart[4],
            Color::CodeKeyword => p.code.keyword,
            Color::CodeString => p.code.string,
            Color::CodeNumber => p.code.number,
            Color::CodeComment => p.code.comment,
            Color::CodeFunction => p.code.function,
            Color::CodeTypeName => p.code.type_name,
            Color::CodeMacroCall => p.code.macro_call,
            Color::CodeLifetime => p.code.lifetime,
            Color::CodeAttribute => p.code.attribute,
        }
    }
}
