//! Role: Public style names of the facade: styles, lengths, colors and themes.
//!
//! The implementation is the `kivact-style` crate; this module re-exports the
//! names an app or a component author calls (`style`, `Style`, `Color`,
//! `Theme`, ...). The drawing vocabulary is `kivact::draw`. The host side of
//! style (`Style::compile`, `Draw::record`, `Theme::install`, ...) stays in
//! `kivact_style` and is not part of the public facade.

#[doc(inline)]
pub use kivact_style::Align;
#[doc(inline)]
pub use kivact_style::{
    Background, BorderStyle, Color, ColorToken, Cursor, Direction, FontFeature, FontKerning,
    GridAutoFlow, GridPlacement, Icon, IconSet, Justify, Length, LineHeight, Overflow, Position,
    ScrollbarWidth, Style, TextAlign, TextDecorationLine, TextLength, TextOverflow, TextTransform,
    Theme, ThemeColor, ThemeExtension, ThemeMode, Track, TransitionProp, auto, current_theme, em,
    hsl, pct, rgb, rgba, style,
};

pub mod theme;
