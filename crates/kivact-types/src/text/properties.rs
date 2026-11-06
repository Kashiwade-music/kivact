//! Role: Authored text content with declared and computed typography.
use crate::text::primitives::{FontKerning, LineHeight, TextAlign, TextOverflow, TextTransform};
use crate::{Fill, FontFeature, Rgba, TextWrap};
use std::ops::Range;

/// Owned text content and optional per-span styling.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextData {
    /// Owned text displayed by this node.
    pub content: Option<String>,
    /// Per-range overrides over the node text.
    pub spans: Option<Vec<TextSpan>>,
}

/// A text length: px, or a multiple of the font size of the text it
/// applies to (CSS `em`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextLen {
    Px(f64),
    Em(f64),
}

impl Default for TextLen {
    fn default() -> Self {
        TextLen::Px(0.0)
    }
}

impl TextLen {
    /// In px, for text of `font_size`.
    pub fn px(self, font_size: f64) -> f64 {
        match self {
            TextLen::Px(v) => v,
            TextLen::Em(v) => v * font_size,
        }
    }
}

/// The pattern used to draw text decorations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextDecorationStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    Wavy,
}

/// Lines drawn with text (CSS `text-decoration`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextDecoration {
    /// The line pattern.
    pub style: TextDecorationStyle,
    /// The lines to draw.
    pub lines: crate::DecorationLines,
    /// `None`: the text's color.
    pub color: Option<Rgba>,
    /// `None`: the font's.
    pub thickness: Option<TextLen>,
    /// CSS `text-underline-offset`: further from the text than the font's
    /// position.
    pub underline_offset: TextLen,
}

/// A shadow of text (CSS `text-shadow`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextShadow {
    pub x: f64,
    pub y: f64,
    /// Blur radius (the gaussian's standard deviation is half of it).
    pub blur: f64,
    pub color: Rgba,
}

/// Declared typography; each absent property inherits from the parent.
/// Capsize is local to this node and does not inherit.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextStyle {
    pub font_size: Option<f64>,
    pub weight: Option<u16>,
    pub color: Option<Rgba>,
    pub line_height: Option<LineHeight>,
    pub wrap: Option<TextWrap>,
    pub align: Option<TextAlign>,
    pub family: Option<String>,
    pub italic: Option<bool>,
    pub letter_spacing: Option<TextLen>,
    pub word_spacing: Option<TextLen>,
    pub kerning: Option<FontKerning>,
    pub features: Option<Vec<FontFeature>>,
    pub tabular_nums: Option<bool>,
    pub decoration_lines: Option<crate::DecorationLines>,
    pub decoration_style: Option<TextDecorationStyle>,
    pub decoration_color: Option<Rgba>,
    pub decoration_thickness: Option<TextLen>,
    pub underline_offset: Option<TextLen>,
    pub overflow: Option<TextOverflow>,
    /// Zero explicitly removes a line clamp inherited from the parent.
    pub max_lines: Option<u32>,
    pub capsize: Option<bool>,
    pub transform: Option<TextTransform>,
    pub shadows: Option<Vec<TextShadow>>,
    pub caret_color: Option<Rgba>,
    pub selection_color: Option<Rgba>,
    pub placeholder_color: Option<Rgba>,
}

/// What a span of rich text sets over its node's text style (`None`: the
/// node's).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpanProps {
    pub font_size: Option<f64>,
    pub weight: Option<u16>,
    pub color: Option<Rgba>,
    pub family: Option<String>,
    pub italic: Option<bool>,
    pub letter_spacing: Option<TextLen>,
    pub word_spacing: Option<TextLen>,
    pub kerning: Option<FontKerning>,
    pub features: Option<Vec<FontFeature>>,
    pub decoration: Option<TextDecoration>,
    /// Filled behind the span's glyphs, one band per line (a `<mark>`).
    pub background: Option<Fill>,
}

/// A byte range of a rich text's `Content` and its style.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub range: Range<usize>,
    pub style: SpanProps,
}
