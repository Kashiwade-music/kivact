//! Role: Everything a text engine computes from its fonts, replaced as a whole.
//! Snapshot keys preserve text, style, width and caret distinctions.

use crate::layouts;
use crate::output;
use kivact_types::render::{RichText, TextSpan};
use kivact_types::{ComputedTextStyle, FontFeature, TextDecoration, TextWrap};
use kivact_types::{
    DecorationLines, FontKerning, LineHeight, Rgba, TextAlign, TextOverflow, TextTransform,
};
use std::collections::HashMap;
use std::hash::Hash;

/// Two-generation map bounded by entry count.
///
/// Hits in the previous generation move to the current one. When the
/// current generation fills, it becomes the previous one and the old
/// previous generation is dropped, so at most `2 * capacity` entries stay.
pub(crate) struct Generations<K, V> {
    current: HashMap<K, V>,
    previous: HashMap<K, V>,
    capacity: usize,
}

impl<K: Hash + Eq, V: Clone> Generations<K, V> {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            current: HashMap::new(),
            previous: HashMap::new(),
            capacity,
        }
    }

    pub(crate) fn get(&mut self, key: K) -> Result<V, K> {
        if let Some(value) = self.current.get(&key) {
            return Ok(value.clone());
        }
        match self.previous.remove(&key) {
            Some(value) => {
                self.insert(key, value.clone());
                Ok(value)
            }
            None => Err(key),
        }
    }

    pub(crate) fn insert(&mut self, key: K, value: V) {
        if self.current.len() >= self.capacity {
            self.previous = std::mem::take(&mut self.current);
        }
        self.current.insert(key, value);
    }
}

fn color_bits(c: Rgba) -> [u64; 4] {
    [c.r, c.g, c.b, c.a].map(f64::to_bits)
}

/// A decoration's pattern, lines, color, thickness and offset, as bits.
type DecorationBits = (u8, u8, Option<[u64; 4]>, Option<(u8, u64)>, (u8, u64));

fn decoration_bits(d: TextDecoration) -> DecorationBits {
    let lines = [
        DecorationLines::UNDERLINE,
        DecorationLines::OVERLINE,
        DecorationLines::LINE_THROUGH,
    ]
    .iter()
    .enumerate()
    .filter(|(_, l)| d.lines.contains(**l))
    .fold(0u8, |bits, (i, _)| bits | (1 << i));
    (
        lines,
        d.style as u8,
        d.color.map(color_bits),
        d.thickness.map(text_len_bits),
        text_len_bits(d.underline_offset),
    )
}

// xtask-allow(state-fields): Immutable cache key; optional style overrides and highlight are independent text properties.
/// What a span sets, as bits; what drawing alone reads (its color,
/// decoration and highlight) is left out for measurement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SpanBits {
    start: usize,
    end: usize,
    font_size: Option<u64>,
    weight: Option<u16>,
    color: Option<[u64; 4]>,
    stroke: Option<(u64, [u64; 4])>,
    family: Option<String>,
    italic: Option<bool>,
    letter_spacing: Option<u64>,
    word_spacing: Option<u64>,
    kerning: Option<FontKerning>,
    features: Option<Vec<FontFeature>>,
    variations: Option<Vec<layouts::VariationBits>>,
    font_width: Option<u64>,
    decoration: Option<DecorationBits>,
    highlight: bool,
}

// xtask-allow(state-fields): Immutable cache key for independent paragraph style options.
/// The rest of the text vocabulary in a key, kept out of line: plain text
/// in a plain style keys none (its key is as small and as quick to hash
/// as before the vocabulary existed).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RichKey {
    letter_spacing: u64,
    word_spacing: u64,
    kerning: FontKerning,
    features: Vec<FontFeature>,
    decoration: Option<DecorationBits>,
    overflow: bool,
    max_lines: Option<u32>,
    capsize: bool,
    transform: TextTransform,
    spans: Vec<SpanBits>,
}

// xtask-allow(state-fields): Immutable cache key for independent text, style, width, and caret options.
/// Exact text, spans, style and wrap width; floats compare by bit
/// pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TextKey {
    text: String,
    font_size: u64,
    weight: u16,
    /// Absent for measurement, which color cannot change.
    color: Option<[u64; 4]>,
    line_height: (u8, u64),
    wrap: bool,
    align: u8,
    family: Option<String>,
    italic: bool,
    /// The paragraph's axis settings, as bits.
    variations: Vec<layouts::VariationBits>,
    font_width: u64,
    optical_sizing: bool,
    max_width: Option<u64>,
    /// Whether the shaped text has its carets.
    carets: bool,
    rich: Option<Box<RichKey>>,
}

impl TextKey {
    /// The key of `text` in `style` at `max_width`; `drawn` keys what only
    /// drawing reads too (colors, decorations, highlights).
    pub(crate) fn new(
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
        drawn: bool,
    ) -> Self {
        let decoration = (drawn && style.decoration != TextDecoration::default())
            .then(|| decoration_bits(style.decoration));
        let rich = (!is_plain(text, style, decoration))
            .then(|| Box::new(rich_key(text, style, decoration, drawn)));
        Self {
            text: text.text.to_owned(),
            font_size: style.font_size.to_bits(),
            weight: style.weight,
            color: drawn.then(|| color_bits(style.color)),
            line_height: line_height_bits(style.line_height),
            wrap: style.wrap == TextWrap::Wrap,
            align: align_bits(style.align),
            family: style.family.clone(),
            italic: style.italic,
            variations: layouts::variation_bits(&style.variations),
            font_width: style.font_width.to_bits(),
            optical_sizing: style.optical_sizing,
            max_width: max_width.map(f64::to_bits),
            carets: true,
            rich,
        }
    }

    /// The key of the same text shaped with or without its carets.
    pub(crate) fn with_carets(mut self, carets: bool) -> Self {
        self.carets = carets;
        self
    }
}

/// Whether a key needs no rich part: plain text in a plain style.
fn is_plain(
    text: RichText<'_>,
    style: &ComputedTextStyle,
    decoration: Option<DecorationBits>,
) -> bool {
    text.spans.is_empty()
        && style.resolve(style.letter_spacing) == 0.0
        && style.resolve(style.word_spacing) == 0.0
        && style.kerning == FontKerning::Auto
        && style.shaping_features().is_empty()
        && decoration.is_none()
        && style.overflow == TextOverflow::Clip
        && style.max_lines.is_none()
        && !style.capsize
        && style.transform == TextTransform::None
}

/// The rich part of a key: the style's vocabulary and every span's bits.
fn rich_key(
    text: RichText<'_>,
    style: &ComputedTextStyle,
    decoration: Option<DecorationBits>,
    drawn: bool,
) -> RichKey {
    RichKey {
        letter_spacing: style.resolve(style.letter_spacing).to_bits(),
        word_spacing: style.resolve(style.word_spacing).to_bits(),
        kerning: style.kerning,
        features: style.shaping_features(),
        decoration,
        overflow: style.overflow == TextOverflow::Ellipsis,
        max_lines: style.max_lines,
        capsize: style.capsize,
        transform: style.transform,
        spans: text
            .spans
            .iter()
            .map(|span| span_bits(span, drawn))
            .collect(),
    }
}

/// One span's bits; drawing-only fields are left out unless `drawn`.
fn span_bits(span: &TextSpan, drawn: bool) -> SpanBits {
    let s = &span.style;
    SpanBits {
        start: span.range.start,
        end: span.range.end,
        font_size: s.font_size.map(f64::to_bits),
        weight: s.weight,
        color: if drawn { s.color.map(color_bits) } else { None },
        stroke: if drawn {
            s.stroke.map(|(w, c)| (w.to_bits(), color_bits(c)))
        } else {
            None
        },
        family: s.family.clone(),
        italic: s.italic,
        letter_spacing: s.letter_spacing.map(f64::to_bits),
        word_spacing: s.word_spacing.map(f64::to_bits),
        kerning: s.kerning,
        features: s.features.clone(),
        variations: s.variations.as_deref().map(layouts::variation_bits),
        font_width: s.font_width.map(f64::to_bits),
        decoration: if drawn {
            s.decoration.map(decoration_bits)
        } else {
            None
        },
        highlight: drawn && s.highlight,
    }
}

fn line_height_bits(line_height: LineHeight) -> (u8, u64) {
    match line_height {
        LineHeight::Normal => (0, 0),
        LineHeight::Scale(v) => (1, v.to_bits()),
        LineHeight::Px(v) => (2, v.to_bits()),
    }
}

fn align_bits(align: TextAlign) -> u8 {
    match align {
        TextAlign::Start => 0,
        TextAlign::Center => 1,
        TextAlign::End => 2,
        TextAlign::Justify => 3,
    }
}

/// Shaped texts kept per cache generation.
const SHAPED_ENTRIES: usize = 1024;
/// Measured extents kept per cache generation.
const MEASURED_ENTRIES: usize = 4096;

// xtask-allow(state-fields): Named text cache; each optional result and collection memoizes an independent computation for the current font generation.
/// Everything one text engine computes from its font collection: replaced as
/// a whole when a font is registered, so no answer of the old collection
/// survives.
pub(crate) struct Cache {
    pub(crate) shaped:
        Generations<TextKey, Result<kivact_types::ShapedTextSnapshot, crate::TextError>>,
    pub(crate) measured: Generations<TextKey, Result<kivact_types::TextMetrics, crate::TextError>>,
    pub(crate) families: HashMap<String, bool>,
    pub(crate) font_names: Option<Vec<String>>,
    /// Shaped paragraphs, broken into lines again for each width.
    pub(crate) paragraphs: layouts::Shaped,
    /// "…" shaped per paragraph style, for ellipses.
    pub(crate) ellipses: HashMap<layouts::Key, output::Ellipsis>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            shaped: Generations::new(SHAPED_ENTRIES),
            measured: Generations::new(MEASURED_ENTRIES),
            families: HashMap::new(),
            font_names: None,
            paragraphs: layouts::Shaped::default(),
            ellipses: HashMap::new(),
        }
    }
}

fn text_len_bits(value: kivact_types::TextLen) -> (u8, u64) {
    match value {
        kivact_types::TextLen::Px(v) => (0, v.to_bits()),
        kivact_types::TextLen::Em(v) => (1, v.to_bits()),
    }
}
