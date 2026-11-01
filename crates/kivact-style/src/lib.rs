//! Role: Styles, lengths, colors, themes and the drawing vocabulary.
//!
//! The implementation behind `kivact::style` and `kivact::draw`: the `Style`
//! sheet, the `Theme` and its tokens, assets, and the `Draw` painter with the
//! paints, shapes, strokes, filters, blending and transforms that styles
//! share with it (they live in `effect`, `paint`, `draw` and `transform`).
//! Each value converts itself into the engine's form next to its definition (a
//! color in the theme it resolves against). The host side is [`Style::compile`]
//! (a whole `Style` into node props), [`draw::Draw::record`] (a drawing into a
//! scene), [`draw::TextRun::from_style`], [`SpanStyle::compile`] and
//! [`Theme::install`] / [`Theme::of`]. This crate owns no node tree, engine,
//! window or renderer; the `kivact-core` facade re-exports the names users call.

#![forbid(unsafe_code)]

pub mod asset;
mod color;
pub mod draw;
pub mod effect;
pub mod geometry;
mod grid;
mod icon;
mod length;
pub mod paint;
pub(crate) mod path;
pub(crate) mod resolve;
mod sheet;
pub mod text;
pub mod theme;
pub mod transform;

pub use color::{Color, ColorToken, hsl, rgb, rgba};
pub use effect::BorderStyle;
pub use grid::{GridAutoFlow, GridPlacement, Track};
pub use icon::{Icon, IconSet};
#[doc(inline)]
pub use kivact_wire::Align;
pub use kivact_wire::{Cursor, TextAlign};
pub use length::{Length, auto, pct};
pub use paint::Background;
pub use resolve::{CompiledStyle, Interactive};
pub use sheet::{
    Direction, Justify, Overflow, Position, ScrollbarWidth, Style, TransitionProp, style,
};
pub use text::{
    FontFeature, FontKerning, LineHeight, SpanStyle, TextDecorationLine, TextLength, TextOverflow,
    TextTransform, em,
};
pub use theme::value::{Theme, ThemeColor, ThemeExtension, ThemeMode, current_theme};

pub(crate) use sheet::paint_transition;
