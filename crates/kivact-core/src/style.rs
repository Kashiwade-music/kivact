//! Role: Public style names of the facade: styles, lengths, colors and themes.
//!
//! The implementation is the `kivact-style` crate; this module re-exports the
//! names an app or a component author calls (`style`, `Style`, `Color`,
//! `Theme`, ...). The drawing vocabulary is `kivact::draw`. The host side of
//! style lives in the hidden `kivact_style::internal` bridge and is not
//! re-exported by the public facade.

#[doc(inline)]
pub use kivact_style::Align;
#[doc(inline)]
pub use kivact_style::{
    Background, BorderStyle, Color, ColorToken, Cursor, Direction, Display, FontFeature,
    FontKerning, GridAutoFlow, GridPlacement, Icon, IconSet, Justify, Length, LineHeight, Overflow,
    Position, ScrollSnap, ScrollbarWidth, Style, StyledRange, TextAlign, TextDecoration,
    TextDecorationLine, TextDecorationStyle, TextLength, TextOverflow, TextTransform, Theme,
    ThemeColor, ThemeExtension, ThemeMode, ThemeRef, Track, TransitionProp, Visibility, auto, em,
    hsl, pct, rgb, rgba, style, use_theme, use_tokens,
};

pub mod theme;

#[doc(inline)]
pub use crate::view::boundary::provide_theme;
