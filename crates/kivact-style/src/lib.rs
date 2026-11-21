//! Role: Styles, lengths, colors, themes and the drawing vocabulary.
//!
//! The implementation behind `kivact::style` (the `Style` sheet, the CSS values, the
//! theme and its variables) and `kivact::draw` (the `Draw` painter and `Path`).
//! Each value converts itself into the engine's form next to its definition (a
//! color in the theme it resolves against). The core host uses the hidden
//! `internal` bridge to compile styles, provide themes and record drawing.
//! This crate owns no node tree, engine, window or renderer; core exports
//! the names users call.

#![forbid(unsafe_code)]

pub mod asset;
pub mod color;
mod color_value;
pub mod control;
pub mod draw;
pub mod effect;
pub mod facade;
pub mod font;
pub mod geometry;
mod grid;
mod icon;
#[doc(hidden)]
pub mod internal;

pub mod paint;
pub(crate) mod path;
pub mod radius;
pub(crate) mod resolve;
pub mod shader;
mod sheet;
pub mod space;
pub mod text;
pub mod theme;
pub mod transform;

pub use color_value::{Color, hsl, hwb, lab, lch, oklab, oklch, rgb, rgba};
pub use effect::BorderStyle;
pub use grid::{GridAutoFlow, GridPlacement, Track};
pub use icon::{Icon, IconSet};
pub use internal::try_use_theme;
#[doc(inline)]
pub use kivact_types::{Align, Display};
#[doc(inline)]
pub use kivact_types::{
    BackfaceVisibility, ColorSpace, HueInterpolation, Isolation, TransformStyle, WillChange,
};
pub use kivact_types::{Cursor, FontStyle, TextAlign};
pub use kivact_types::{Length, auto, em, pct};
pub use kivact_types::{ThemeVar, VarValue};
pub use paint::Background;
pub use resolve::{CompiledStyle, Interactive};
pub use sheet::{
    FlexDirection, Justify, Overflow, Position, ScrollbarWidth, Style, TransitionProp, style,
};
pub use text::{
    FontFeature, FontKerning, LineHeight, SpanStyle, StyledRange, TextDecoration,
    TextDecorationLine, TextDecorationStyle, TextOverflow, TextTransform,
};
pub use theme::value::{Theme, ThemeExtension, ThemeMode, ThemeRef, use_theme};
pub use theme::{FontFamily, ThemeValue};

/// Whether a layout box and its subtree are painted and take input (CSS `visibility`).
pub use kivact_types::Visibility;

pub use kivact_types::ScrollSnap;
