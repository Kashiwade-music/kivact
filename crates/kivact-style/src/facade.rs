//! Role: Curated application style vocabulary forwarded by the public facade.
//!
//! This bounded export list is shared by `kivact::style`. Drawing, backend and
//! internal implementation names stay outside that vocabulary.

#[doc(inline)]
pub use crate::Align;
#[doc(inline)]
pub use crate::{
    Background, BorderStyle, Color, ColorToken, Cursor, Direction, Display, FontFeature,
    FontKerning, GridAutoFlow, GridPlacement, Icon, IconSet, Justify, Length, LineHeight, Overflow,
    Position, ScrollSnap, ScrollbarWidth, Style, StyledRange, TextAlign, TextDecoration,
    TextDecorationLine, TextDecorationStyle, TextLength, TextOverflow, TextTransform, Theme,
    ThemeColor, ThemeExtension, ThemeMode, ThemeRef, Track, TransitionProp, Visibility, auto, em,
    hsl, pct, rgb, rgba, style, use_theme, use_tokens,
};
