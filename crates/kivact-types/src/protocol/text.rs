//! Role: Rich text and text styles a host hands Arvane to shape.
//! Plain data usable on any thread.

use crate::color::Rgba;
use crate::text::primitives::{
    DecorationLines, FontKerning, LineHeight, TextAlign, TextOverflow, TextTransform,
};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Textwrap.
pub enum TextWrap {
    #[default]
    /// Wrap.
    Wrap,
    /// Nowrap.
    NoWrap,
}

/// An OpenType feature setting (CSS `font-feature-settings: "tnum" 1`):
/// a four byte tag and its value (`1` on, `0` off, or an alternate's
/// index).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontFeature {
    /// The feature's tag (`*b"tnum"`).
    pub tag: [u8; 4],
    /// Its value.
    pub value: u16,
}

impl FontFeature {
    /// The feature `tag` set to `value`.
    pub const fn new(tag: [u8; 4], value: u16) -> Self {
        Self { tag, value }
    }
}

/// Lines drawn with text (CSS `text-decoration`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextDecoration {
    /// Which lines.
    pub lines: DecorationLines,
    /// Their color; `None` is the text's color (CSS `currentColor`).
    pub color: Option<Rgba>,
    /// Their thickness in px; `None` is the font's.
    pub thickness: Option<f32>,
    /// How much further from the text than the font's position the
    /// underline lies, in px (CSS `text-underline-offset`; negative moves
    /// it up).
    pub underline_offset: f32,
}

/// Text style of a paragraph; inherited by descendant text in the host.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    /// Font size.
    pub font_size: f32,
    /// Weight (any value from 1 to 1000: a variable font's `wght` axis
    /// takes it as it is).
    pub weight: u16,
    /// Color.
    pub color: Rgba,
    /// Line height.
    pub line_height: LineHeight,
    /// Wrap.
    pub wrap: TextWrap,
    /// Align.
    pub align: TextAlign,
    /// A family name or an ordered CSS family list, tried per glyph before
    /// the system UI fallback (for example `Inter, 'Noto Sans JP'`).
    pub family: Option<String>,
    /// Italic: the family's italic face when it has one, else the text
    /// system's slanted upright face. Defaults to false.
    pub italic: bool,
    /// Extra space after every character, in px (CSS `letter-spacing`).
    pub letter_spacing: f32,
    /// Extra space after every space, in px (CSS `word-spacing`).
    pub word_spacing: f32,
    /// Pair kerning (CSS `font-kerning`). It sets the `kern` feature
    /// before `features`, which can still override it.
    pub kerning: FontKerning,
    /// OpenType features (CSS `font-feature-settings`).
    pub features: Vec<FontFeature>,
    /// Lines drawn with the text.
    pub decoration: TextDecoration,
    /// What a line that does not fit the width shows.
    pub overflow: TextOverflow,
    /// The most lines shown (CSS `line-clamp`): the last one shown ends
    /// in "…" when more follow. `None` shows them all.
    pub max_lines: Option<u32>,
    /// Trim the first line to its cap height and the last to its baseline.
    pub capsize: bool,
    /// Case conversion when shaping.
    pub transform: TextTransform,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font_size: 14.0,
            weight: 400,
            color: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            line_height: LineHeight::Normal,
            wrap: TextWrap::Wrap,
            align: TextAlign::Start,
            family: None,
            italic: false,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            kerning: FontKerning::Auto,
            features: Vec::new(),
            decoration: TextDecoration::default(),
            overflow: TextOverflow::Clip,
            max_lines: None,
            capsize: false,
            transform: TextTransform::None,
        }
    }
}

impl TextStyle {
    /// Whether a line that does not fit is cut with "…" (an ellipsis on
    /// one line, or lines clamped).
    pub fn ellipsizes(&self) -> bool {
        self.overflow == TextOverflow::Ellipsis || self.max_lines.is_some()
    }
}

/// What a span of rich text sets over its paragraph's [`TextStyle`]
/// (`None`: the paragraph's).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpanStyle {
    /// Font size.
    pub font_size: Option<f32>,
    /// Weight.
    pub weight: Option<u16>,
    /// Color of its glyphs.
    pub color: Option<Rgba>,
    /// A family name or an ordered CSS family list, overriding the paragraph.
    pub family: Option<String>,
    /// Italic.
    pub italic: Option<bool>,
    /// Extra space after every character, in px.
    pub letter_spacing: Option<f32>,
    /// Extra space after every space, in px.
    pub word_spacing: Option<f32>,
    /// Pair kerning.
    pub kerning: Option<FontKerning>,
    /// OpenType features (replacing the paragraph's).
    pub features: Option<Vec<FontFeature>>,
    /// Lines drawn with it (replacing the paragraph's).
    pub decoration: Option<TextDecoration>,
    /// Its band (one rect per line it is on) is reported in
    /// [`crate::protocol::ShapedText::highlights`], for the host to fill behind the
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
