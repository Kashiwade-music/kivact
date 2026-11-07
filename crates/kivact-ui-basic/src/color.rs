//! Role: The basic kit's own colors as values a style or a draw call accepts.

use crate::plot::PlotColor;
use crate::tokens::Tokens;
use kivact::style::{Color, Theme, ThemeColor, rgb};

/// A color of the Basic kit's [`Tokens`], read from the theme it is painted in.
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

impl ThemeColor for ColorName {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let p = theme.ext::<Tokens>();
        match self {
            ColorName::Background => p.background,
            ColorName::Foreground => p.foreground,
            ColorName::Card => p.card,
            ColorName::CardForeground => p.card_foreground,
            ColorName::Popover => p.popover,
            ColorName::PopoverForeground => p.popover_foreground,
            ColorName::Primary => p.primary,
            ColorName::PrimaryForeground => p.primary_foreground,
            ColorName::Secondary => p.secondary,
            ColorName::SecondaryForeground => p.secondary_foreground,
            ColorName::Muted => p.muted,
            ColorName::MutedForeground => p.muted_foreground,
            ColorName::Accent => p.accent,
            ColorName::AccentForeground => p.accent_foreground,
            ColorName::Destructive => p.destructive,
            ColorName::Border => p.border,
            ColorName::Input => p.input,
            ColorName::Ring => p.ring,
            ColorName::Sidebar => p.sidebar,
            ColorName::SidebarForeground => p.sidebar_foreground,
            ColorName::SidebarPrimary => p.sidebar_primary,
            ColorName::SidebarPrimaryForeground => p.sidebar_primary_foreground,
            ColorName::SidebarAccent => p.sidebar_accent,
            ColorName::SidebarAccentForeground => p.sidebar_accent_foreground,
            ColorName::SidebarBorder => p.sidebar_border,
            ColorName::SidebarRing => p.sidebar_ring,
            ColorName::Chart1 => p.chart[0],
            ColorName::Chart2 => p.chart[1],
            ColorName::Chart3 => p.chart[2],
            ColorName::Chart4 => p.chart[3],
            ColorName::Chart5 => p.chart[4],
            ColorName::CodeKeyword => p.code.keyword,
            ColorName::CodeString => p.code.string,
            ColorName::CodeNumber => p.code.number,
            ColorName::CodeComment => p.code.comment,
            ColorName::CodeFunction => p.code.function,
            ColorName::CodeTypeName => p.code.type_name,
            ColorName::CodeMacroCall => p.code.macro_call,
            ColorName::CodeLifetime => p.code.lifetime,
            ColorName::CodeAttribute => p.code.attribute,
        }
    }
}

/// The color of a plot series: a chart color of `tokens`, or a fixed RGB value.
pub(crate) fn series_color(color: PlotColor, tokens: &Tokens) -> Color {
    match color {
        PlotColor::Palette(i) => tokens.chart[i as usize % tokens.chart.len()],
        PlotColor::Rgb(value) => rgb(value),
    }
}
