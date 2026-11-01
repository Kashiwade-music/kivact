//! Role: Text rendering and measurement.
//!
//! Text style values: the style of a paragraph ([`TextStyle`]), the spans
//! of rich text that override it ([`TextSpan`], [`RichText`]) and what
//! shaping draws with the glyphs ([`TextDecorationRect`],
//! [`TextHighlight`]). Plain data usable on any thread.

use std::ops::{BitOr, BitOrAssign, Range};

use crate::{Rect, Rgba, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Textwrap.
pub enum TextWrap {
    #[default]
    /// Wrap.
    Wrap,
    /// Nowrap.
    NoWrap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Textalign.
pub enum TextAlign {
    #[default]
    /// Start.
    Start,
    /// Center.
    Center,
    /// End.
    End,
    /// Every line but the last of a paragraph spans the width (CSS
    /// `text-align: justify`) by widening its spaces; the last line, and
    /// every line of text shaped without a width, starts at the start edge.
    Justify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
/// Pair kerning (CSS `font-kerning`).
pub enum FontKerning {
    #[default]
    /// The font's default.
    Auto,
    /// The OpenType `kern` feature on.
    Normal,
    /// The OpenType `kern` feature off.
    None,
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

/// The lines drawn with text (CSS `text-decoration-line`), combined with
/// `|`: `DecorationLines::UNDERLINE | DecorationLines::OVERLINE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct DecorationLines(u8);

impl DecorationLines {
    /// No line.
    pub const NONE: Self = Self(0);
    /// A line under the text, drawn before the glyphs.
    pub const UNDERLINE: Self = Self(1);
    /// A line over the text, drawn before the glyphs.
    pub const OVERLINE: Self = Self(2);
    /// A line through the text, drawn after the glyphs.
    pub const LINE_THROUGH: Self = Self(4);

    /// Whether every line of `other` is in `self`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether no line is drawn.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for DecorationLines {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DecorationLines {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
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

/// What text that does not fit shows (CSS `text-overflow`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextOverflow {
    /// It overflows (the default).
    #[default]
    Clip,
    /// The last line shown ends in "…" where it would leave the width.
    Ellipsis,
}

/// Case conversion applied when text is shaped (CSS `text-transform`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextTransform {
    /// The text as it is.
    #[default]
    None,
    /// Every letter in upper case.
    Uppercase,
    /// Every letter in lower case.
    Lowercase,
    /// The first letter of each word in upper case (a word follows the
    /// start or a white space).
    Capitalize,
}

/// The distance between the baselines of a paragraph's lines (CSS
/// `line-height`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum LineHeight {
    /// `round(font_size * 1.3)` (the default).
    #[default]
    Normal,
    /// A multiple of the font size (CSS `line-height: 1.5`).
    Scale(f32),
    /// Logical px (CSS `line-height: 20px`).
    Px(f32),
}

impl LineHeight {
    /// A line height in logical pixels, independent of the font size.
    pub fn px(v: impl Into<f64>) -> Self {
        LineHeight::Px(v.into() as f32)
    }
}

impl From<i32> for LineHeight {
    fn from(v: i32) -> Self {
        LineHeight::Scale(v as f32)
    }
}

impl From<u32> for LineHeight {
    fn from(v: u32) -> Self {
        LineHeight::Scale(v as f32)
    }
}

impl From<f32> for LineHeight {
    fn from(v: f32) -> Self {
        LineHeight::Scale(v)
    }
}

impl From<f64> for LineHeight {
    fn from(v: f64) -> Self {
        LineHeight::Scale(v as f32)
    }
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
    /// [`crate::ShapedText::highlights`], for the host to fill behind the
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

/// A line drawn with shaped text, in the text's coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextDecorationRect {
    /// Where.
    pub rect: Rect,
    /// Its color.
    pub color: Rgba,
    /// Which line it is (one of them): a line-through is drawn after the
    /// glyphs, an underline or overline before them.
    pub line: DecorationLines,
}

/// The band of a highlighted span on one line, in the text's coordinates
/// (the span's glyphs' advance by their font's ascent and descent).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextHighlight {
    /// Where.
    pub rect: Rect,
    /// The span's index in [`RichText::spans`].
    pub span: usize,
}

/// The extent of measured text.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMetrics {
    /// Its width and height.
    pub size: Size,
    /// Its first line's baseline, from the top (`0` for empty text).
    pub baseline: f32,
}
