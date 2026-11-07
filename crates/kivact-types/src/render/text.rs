//! Role: Rich text and text styles a host hands Arvane to shape.
//! Plain data usable on any thread.

use crate::color::Rgba;
use crate::text::{FontFeature, FontKerning, FontVariation, TextDecoration};
use std::ops::Range;

/// What a span of rich text sets over its paragraph's [`crate::text::ComputedTextStyle`]
/// (`None`: the paragraph's).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpanStyle {
    /// Font size.
    pub font_size: Option<f64>,
    /// Weight.
    pub weight: Option<u16>,
    /// Color of its glyphs.
    pub color: Option<Rgba>,
    /// A family name or an ordered CSS family list, overriding the paragraph.
    pub family: Option<String>,
    /// Italic.
    pub italic: Option<bool>,
    /// Extra space after every character, in px.
    pub letter_spacing: Option<f64>,
    /// Extra space after every space, in px.
    pub word_spacing: Option<f64>,
    /// Pair kerning.
    pub kerning: Option<FontKerning>,
    /// OpenType features (replacing the paragraph's).
    pub features: Option<Vec<FontFeature>>,
    /// Lines drawn with it (replacing the paragraph's).
    pub decoration: Option<TextDecoration>,
    /// Variable-font axis settings (replacing the paragraph's).
    pub variations: Option<Vec<FontVariation>>,
    /// CSS font-width (font-stretch) in percent (replacing the paragraph's).
    pub font_width: Option<f64>,
    /// Its band (one rect per line it is on) is reported in
    /// [`crate::render::ShapedText::highlights`], for the host to fill behind the
    /// glyphs (a `<mark>`).
    pub highlight: bool,
}

/// A range of rich text and its style.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    /// Half-open UTF-8 byte range of the original input text, on Unicode
    /// scalar boundaries. Case conversion remaps it before shaping.
    pub range: Range<usize>,
    /// What it sets.
    pub style: SpanStyle,
}

/// A paragraph: its text and its spans (sorted, not overlapping; text
/// outside them has the paragraph's style). A `&str` is a paragraph
/// without spans.
/// Both slices are borrowed for a shaping or measurement call; returned
/// shaping snapshots retain no borrow of either one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RichText<'a> {
    /// The text.
    pub text: &'a str,
    /// Its spans.
    pub spans: &'a [TextSpan],
}

impl<'a> From<&'a str> for RichText<'a> {
    fn from(text: &'a str) -> Self {
        RichText { text, spans: &[] }
    }
}

impl<'a> From<&'a String> for RichText<'a> {
    fn from(text: &'a String) -> Self {
        RichText { text, spans: &[] }
    }
}
