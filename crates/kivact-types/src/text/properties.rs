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

/// A text length: px (a number: `letter_spacing(1)`) or a multiple of the
/// font size of the text it applies to (CSS `0.08em` is `TextLength::Em(0.08)`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextLength {
    /// Logical px.
    Px(f64),
    /// A multiple of the text's font size.
    Em(f64),
}

impl Default for TextLength {
    fn default() -> Self {
        TextLength::Px(0.0)
    }
}

impl From<i32> for TextLength {
    fn from(v: i32) -> Self {
        TextLength::Px(v as f64)
    }
}

impl From<u32> for TextLength {
    fn from(v: u32) -> Self {
        TextLength::Px(v as f64)
    }
}

impl From<f32> for TextLength {
    fn from(value: f32) -> Self {
        Self::Px(f64::from(value))
    }
}

impl From<f64> for TextLength {
    fn from(v: f64) -> Self {
        TextLength::Px(v)
    }
}

impl TextLength {
    /// In px, for text of `font_size`.
    pub fn px(self, font_size: f64) -> f64 {
        match self {
            TextLength::Px(v) => v,
            TextLength::Em(v) => v * font_size,
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
    pub thickness: Option<TextLength>,
    /// CSS `text-underline-offset`: further from the text than the font's
    /// position.
    pub underline_offset: TextLength,
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

/// An outline drawn around glyphs (CSS `-webkit-text-stroke`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStroke {
    /// The outline width, centered on the glyph edge.
    pub width: TextLength,
    /// The outline color.
    pub color: Rgba,
}

/// Whether the fill or the stroke of glyphs is painted first (CSS
/// `paint-order`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PaintOrder {
    /// The fill is painted first, then the stroke over it.
    #[default]
    FillStroke,
    /// The stroke is painted first, then the fill over it.
    StrokeFill,
}

/// A variable-font axis setting (CSS `font-variation-settings`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontVariation {
    /// The four-byte OpenType axis tag, such as `*b"wght"`.
    pub tag: [u8; 4],
    /// The axis value.
    pub value: f64,
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
    pub letter_spacing: Option<TextLength>,
    pub word_spacing: Option<TextLength>,
    pub kerning: Option<FontKerning>,
    pub features: Option<Vec<FontFeature>>,
    pub tabular_nums: Option<bool>,
    pub decoration_lines: Option<crate::DecorationLines>,
    pub decoration_style: Option<TextDecorationStyle>,
    pub decoration_color: Option<Rgba>,
    pub decoration_thickness: Option<TextLength>,
    pub underline_offset: Option<TextLength>,
    pub overflow: Option<TextOverflow>,
    /// Zero explicitly removes a line clamp inherited from the parent.
    pub max_lines: Option<u32>,
    pub capsize: Option<bool>,
    pub transform: Option<TextTransform>,
    pub shadows: Option<Vec<TextShadow>>,
    pub caret_color: Option<Rgba>,
    pub selection_color: Option<Rgba>,
    pub placeholder_color: Option<Rgba>,
    /// An outline around glyphs.
    pub stroke: Option<TextStroke>,
    /// Whether the fill or the stroke is painted first.
    pub paint_order: Option<PaintOrder>,
    /// Variable-font axis settings.
    pub variations: Option<Vec<FontVariation>>,
    /// Whether the optical-size axis follows the font size (CSS
    /// `font-optical-sizing`).
    pub optical_sizing: Option<bool>,
    /// CSS font-width (font-stretch) in percent, 50..=200.
    pub font_width: Option<f64>,
}

/// A byte range of a rich text's `Content` and what it sets over the
/// paragraph's text style (`None` fields are the paragraph's).
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub range: Range<usize>,
    /// The text properties it sets.
    pub style: TextStyle,
    /// Filled behind the span's glyphs, one band per line (a `<mark>`).
    pub background: Option<Fill>,
}

impl TextStyle {
    /// The decoration a span or an editing range sets over its paragraph's:
    /// `None` when it sets none of the decoration properties.
    pub fn span_decoration(&self) -> Option<TextDecoration> {
        let decorated = self.decoration_lines.is_some()
            || self.decoration_color.is_some()
            || self.decoration_thickness.is_some()
            || self.underline_offset.is_some()
            || self.decoration_style.is_some();
        decorated.then(|| TextDecoration {
            style: self.decoration_style.unwrap_or_default(),
            lines: self.decoration_lines.unwrap_or_default(),
            color: self.decoration_color,
            thickness: self.decoration_thickness,
            underline_offset: self.underline_offset.unwrap_or_default(),
        })
    }
}
