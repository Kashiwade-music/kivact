//! Role: Text alignment, kerning, overflow, case, line height and metrics.
//!
//! Plain data usable on any thread.

use crate::geometry::Size;
use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
/// Textalign.
#[non_exhaustive]
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
#[non_exhaustive]
pub enum FontKerning {
    #[default]
    /// The font's default.
    Auto,
    /// The OpenType `kern` feature on.
    Normal,
    /// The OpenType `kern` feature off.
    None,
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

/// What text that does not fit shows (CSS `text-overflow`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum TextOverflow {
    /// It overflows (the default).
    #[default]
    Clip,
    /// The last line shown ends in "…" where it would leave the width.
    Ellipsis,
}

/// Case conversion applied when text is shaped (CSS `text-transform`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
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
#[non_exhaustive]
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

/// What measuring text answers (`Draw::measure_text`,
/// `Constraints::measure_text`): its extent and where its first baseline
/// lies, in logical pixels. Empty text measures `TextMetrics::default()`.
/// Built with [`TextMetrics::new`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TextMetrics {
    /// The width and height of the measured text.
    pub size: Size,
    /// The first line's baseline, from the top of the text (`0` for empty
    /// text). Place the text at `y - baseline` to put the baseline at `y`.
    pub baseline: f32,
}

impl TextMetrics {
    /// Metrics of text of extent `size` whose first baseline is `baseline`
    /// below its top.
    pub const fn new(size: Size, baseline: f32) -> Self {
        TextMetrics { size, baseline }
    }
}
