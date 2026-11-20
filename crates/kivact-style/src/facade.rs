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
    BackfaceVisibility, Background, BorderStyle, Color, ColorSpace, Cursor, Direction, Display,
    FontFamily, FontFeature, FontKerning, GridAutoFlow, GridPlacement, HueInterpolation, Icon,
    IconSet, Isolation, Justify, Length, LineHeight, Overflow, Position, ScrollSnap,
    ScrollbarWidth, Style, StyledRange, TextAlign, TextDecoration, TextDecorationLine,
    TextDecorationStyle, TextOverflow, TextTransform, Theme, ThemeExtension, ThemeMode, ThemeRef,
    ThemeValue, ThemeVar, Track, TransformStyle, TransitionProp, VarValue, Visibility, WillChange,
    auto, color, control, em, font, hsl, hwb, lab, lch, oklab, oklch, pct, radius, rgb, rgba,
    space, style, try_use_theme, use_theme,
};
