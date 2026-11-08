//! Role: Curated application style vocabulary forwarded by the public facade.
//!
//! This bounded export list is shared by `kivact::style`. Drawing, backend and
//! internal implementation names stay outside that vocabulary.

#[doc(inline)]
pub use crate::Align;
#[doc(inline)]
pub use crate::paint::{BorderImage, BorderImageRepeat};
#[doc(inline)]
pub use crate::text::{FontVariation, PaintOrder};
#[doc(inline)]
pub use crate::{
    BackfaceVisibility, Background, BorderStyle, Color, ColorSpace, ColorToken, Cursor, Direction,
    Display, FontFeature, FontKerning, GridAutoFlow, GridPlacement, HueInterpolation, Icon,
    IconSet, Isolation, Justify, Length, LineHeight, Overflow, Position, ScrollSnap,
    ScrollbarWidth, Style, StyledRange, TextAlign, TextDecoration, TextDecorationLine,
    TextDecorationStyle, TextLength, TextOverflow, TextTransform, Theme, ThemeColor,
    ThemeExtension, ThemeMode, ThemeRef, Track, TransformStyle, TransitionProp, Visibility,
    WillChange, auto, em, hsl, hwb, lab, lch, oklab, oklch, pct, rgb, rgba, style, use_theme,
    use_tokens,
};
