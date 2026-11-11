//! Role: Everything a text engine computes from its fonts, replaced as a whole.
//! Snapshot keys preserve text, style, width and caret distinctions.

use crate::layouts;
use crate::output;
use hashbrown::{Equivalent, HashMap as LookupMap};
use kivact_types::render::{RichText, TextSpan};
use kivact_types::{ComputedTextStyle, FontFeature, FontVariation, TextDecoration, TextWrap};
use kivact_types::{
    DecorationLines, FontKerning, LineHeight, Rgba, TextAlign, TextOverflow, TextTransform,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Two-generation map bounded by entry count.
///
/// Hits in the previous generation move to the current one. When the
/// current generation fills, it becomes the previous one and the old
/// previous generation is dropped, so at most `2 * capacity` entries stay.
/// A lookup borrows its key (`Equivalent`), so only a miss allocates one.
pub(crate) struct Generations<K, V> {
    current: LookupMap<K, V>,
    previous: LookupMap<K, V>,
    capacity: usize,
}

impl<K: Hash + Eq, V: Clone> Generations<K, V> {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            current: LookupMap::new(),
            previous: LookupMap::new(),
            capacity,
        }
    }

    /// The value of the entry that `key` names; a hit in the previous
    /// generation moves to the current one (it keeps its owned key).
    pub(crate) fn get<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        if let Some(value) = self.current.get(key) {
            return Some(value.clone());
        }
        let (key, value) = self.previous.remove_entry(key)?;
        self.insert(key, value.clone());
        Some(value)
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

/// A variable-font axis setting as bits, borrowed for a lookup; it hashes
/// and compares as the `Vec<VariationBits>` that [`TextKey`] and
/// [`RichKey`] keep.
#[derive(Clone, Copy)]
struct VariationsRef<'a>(&'a [FontVariation]);

impl Hash for VariationsRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.len().hash(state);
        for variation in self.0 {
            (variation.tag, variation.value.to_bits()).hash(state);
        }
    }
}

impl VariationsRef<'_> {
    /// Whether these axis settings are `bits`.
    fn is(&self, bits: &[layouts::VariationBits]) -> bool {
        self.0.len() == bits.len()
            && self.0.iter().zip(bits).all(|(variation, (tag, value))| {
                variation.tag == *tag && variation.value.to_bits() == *value
            })
    }
}

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

/// What a span sets, as bits; what drawing alone reads (its color,
/// decoration and highlight) is left out for measurement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SpanBits {
    start: usize,
    end: usize,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    font_size: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    weight: Option<u16>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    color: Option<[u64; 4]>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    stroke: Option<(u64, [u64; 4])>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    family: Option<String>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    italic: Option<bool>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    letter_spacing: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    word_spacing: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    kerning: Option<FontKerning>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    features: Option<Vec<FontFeature>>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    variations: Option<Vec<layouts::VariationBits>>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    font_width: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    decoration: Option<DecorationBits>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    highlight: bool,
}

/// What a span sets, borrowed for a lookup; it hashes and compares as the
/// [`SpanBits`] the cache keeps.
#[derive(Hash)]
struct SpanBitsRef<'a> {
    start: usize,
    end: usize,
    // xtask-allow(state-fields): cache: one component of a cache key
    font_size: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key
    weight: Option<u16>,
    // xtask-allow(state-fields): cache: one component of a cache key
    color: Option<[u64; 4]>,
    // xtask-allow(state-fields): cache: one component of a cache key
    stroke: Option<(u64, [u64; 4])>,
    // xtask-allow(state-fields): cache: one component of a cache key
    family: Option<&'a str>,
    // xtask-allow(state-fields): cache: one component of a cache key
    italic: Option<bool>,
    // xtask-allow(state-fields): cache: one component of a cache key
    letter_spacing: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key
    word_spacing: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key
    kerning: Option<FontKerning>,
    // xtask-allow(state-fields): cache: one component of a cache key
    features: Option<&'a [FontFeature]>,
    // xtask-allow(state-fields): cache: one component of a cache key
    variations: Option<VariationsRef<'a>>,
    // xtask-allow(state-fields): cache: one component of a cache key
    font_width: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key
    decoration: Option<DecorationBits>,
    // xtask-allow(state-fields): cache: one component of a cache key
    highlight: bool,
}

impl Equivalent<SpanBits> for SpanBitsRef<'_> {
    fn equivalent(&self, other: &SpanBits) -> bool {
        self.start == other.start
            && self.end == other.end
            && self.font_size == other.font_size
            && self.weight == other.weight
            && self.color == other.color
            && self.stroke == other.stroke
            && self.family == other.family.as_deref()
            && self.italic == other.italic
            && self.letter_spacing == other.letter_spacing
            && self.word_spacing == other.word_spacing
            && self.kerning == other.kerning
            && same_slice(self.features, other.features.as_deref())
            && same_variations(self.variations, other.variations.as_deref())
            && self.font_width == other.font_width
            && self.decoration == other.decoration
            && self.highlight == other.highlight
    }
}

/// Whether two optional lists are both absent or both equal.
fn same_slice<T: PartialEq>(a: Option<&[T]>, b: Option<&[T]>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a == b,
        (None, None) => true,
        _ => false,
    }
}

/// Whether two optional axis-setting lists are both absent or both equal.
fn same_variations(a: Option<VariationsRef<'_>>, b: Option<&[layouts::VariationBits]>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.is(b),
        (None, None) => true,
        _ => false,
    }
}

/// The rest of the text vocabulary in a key, kept out of line: plain text
/// in a plain style keys none (its key is the text and the common style only,
/// small and quick to hash).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RichKey {
    letter_spacing: u64,
    word_spacing: u64,
    kerning: FontKerning,
    features: Vec<FontFeature>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    decoration: Option<DecorationBits>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    overflow: bool,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    max_lines: Option<u32>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    capsize: bool,
    transform: TextTransform,
    spans: Vec<SpanBits>,
}

/// Exact text, spans, style and wrap width; floats compare by bit
/// pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TextKey {
    text: String,
    font_size: u64,
    weight: u16,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    /// Absent for measurement, which color cannot change.
    color: Option<[u64; 4]>,
    line_height: (u8, u64),
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    wrap: bool,
    align: u8,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    family: Option<String>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    italic: bool,
    /// The paragraph's axis settings, as bits.
    variations: Vec<layouts::VariationBits>,
    font_width: u64,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    optical_sizing: bool,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    max_width: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    /// Whether the shaped text has its carets.
    carets: bool,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    rich: Option<Box<RichKey>>,
}

/// A borrowed [`TextKey`], hashing and comparing the same way, so that a
/// lookup allocates nothing (`Equivalent`).
#[derive(Hash)]
pub(crate) struct TextKeyRef<'a> {
    text: &'a str,
    font_size: u64,
    weight: u16,
    // xtask-allow(state-fields): cache: one component of a cache key
    color: Option<[u64; 4]>,
    line_height: (u8, u64),
    // xtask-allow(state-fields): cache: one component of a cache key
    wrap: bool,
    align: u8,
    // xtask-allow(state-fields): cache: one component of a cache key
    family: Option<&'a str>,
    // xtask-allow(state-fields): cache: one component of a cache key
    italic: bool,
    variations: VariationsRef<'a>,
    font_width: u64,
    // xtask-allow(state-fields): cache: one component of a cache key
    optical_sizing: bool,
    // xtask-allow(state-fields): cache: one component of a cache key
    max_width: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key
    carets: bool,
    // xtask-allow(state-fields): cache: one component of a cache key
    rich: Option<RichKeyRef<'a>>,
}

impl<'a> TextKeyRef<'a> {
    /// The key of `text` in `style` at `max_width`; `drawn` keys what only
    /// drawing reads too (colors, decorations, highlights).
    pub(crate) fn new(
        text: RichText<'a>,
        style: &'a ComputedTextStyle,
        max_width: Option<f64>,
        drawn: bool,
    ) -> Self {
        let decoration = (drawn && style.decoration != TextDecoration::default())
            .then(|| decoration_bits(style.decoration));
        let rich =
            (!is_plain(text, style, decoration)).then(|| rich_key(text, style, decoration, drawn));
        Self {
            text: text.text,
            font_size: style.font_size.to_bits(),
            weight: style.weight,
            color: drawn.then(|| color_bits(style.color)),
            line_height: line_height_bits(style.line_height),
            wrap: style.wrap == TextWrap::Wrap,
            align: align_bits(style.align),
            family: style.family.as_deref(),
            italic: style.italic,
            variations: VariationsRef(&style.variations),
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

    /// The same key, owning its text and style parts.
    pub(crate) fn to_owned(&self) -> TextKey {
        TextKey {
            text: self.text.to_owned(),
            font_size: self.font_size,
            weight: self.weight,
            color: self.color,
            line_height: self.line_height,
            wrap: self.wrap,
            align: self.align,
            family: self.family.map(str::to_owned),
            italic: self.italic,
            variations: layouts::variation_bits(self.variations.0),
            font_width: self.font_width,
            optical_sizing: self.optical_sizing,
            max_width: self.max_width,
            carets: self.carets,
            rich: self.rich.as_ref().map(|rich| Box::new(rich.to_owned())),
        }
    }
}

impl Equivalent<TextKey> for TextKeyRef<'_> {
    fn equivalent(&self, other: &TextKey) -> bool {
        self.text == other.text
            && self.font_size == other.font_size
            && self.weight == other.weight
            && self.color == other.color
            && self.line_height == other.line_height
            && self.wrap == other.wrap
            && self.align == other.align
            && self.family == other.family.as_deref()
            && self.italic == other.italic
            && self.variations.is(&other.variations)
            && self.font_width == other.font_width
            && self.optical_sizing == other.optical_sizing
            && self.max_width == other.max_width
            && self.carets == other.carets
            && match (&self.rich, &other.rich) {
                (Some(a), Some(b)) => a.equivalent(b),
                (None, None) => true,
                _ => false,
            }
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
        && style.features.is_empty()
        && !style.tabular_nums
        && decoration.is_none()
        && style.overflow == TextOverflow::Clip
        && style.max_lines.is_none()
        && !style.capsize
        && style.transform == TextTransform::None
}

/// The rich part of a key, borrowed for a lookup; it hashes and compares as
/// the [`RichKey`] the cache keeps.
#[derive(Hash)]
struct RichKeyRef<'a> {
    letter_spacing: u64,
    word_spacing: u64,
    kerning: FontKerning,
    features: FeaturesRef<'a>,
    // xtask-allow(state-fields): cache: one component of a cache key
    decoration: Option<DecorationBits>,
    // xtask-allow(state-fields): cache: one component of a cache key
    overflow: bool,
    // xtask-allow(state-fields): cache: one component of a cache key
    max_lines: Option<u32>,
    // xtask-allow(state-fields): cache: one component of a cache key
    capsize: bool,
    transform: TextTransform,
    spans: SpansRef<'a>,
}

/// The rich part of a key: the style's vocabulary and every span's bits.
fn rich_key<'a>(
    text: RichText<'a>,
    style: &'a ComputedTextStyle,
    decoration: Option<DecorationBits>,
    drawn: bool,
) -> RichKeyRef<'a> {
    RichKeyRef {
        letter_spacing: style.resolve(style.letter_spacing).to_bits(),
        word_spacing: style.resolve(style.word_spacing).to_bits(),
        kerning: style.kerning,
        features: FeaturesRef(style),
        decoration,
        overflow: style.overflow == TextOverflow::Ellipsis,
        max_lines: style.max_lines,
        capsize: style.capsize,
        transform: style.transform,
        spans: SpansRef {
            spans: text.spans,
            drawn,
        },
    }
}

impl RichKeyRef<'_> {
    /// The same key, owning its features and span bits.
    fn to_owned(&self) -> RichKey {
        RichKey {
            letter_spacing: self.letter_spacing,
            word_spacing: self.word_spacing,
            kerning: self.kerning,
            features: self.features.to_owned(),
            decoration: self.decoration,
            overflow: self.overflow,
            max_lines: self.max_lines,
            capsize: self.capsize,
            transform: self.transform,
            spans: self.spans.to_owned(),
        }
    }
}

impl Equivalent<RichKey> for RichKeyRef<'_> {
    fn equivalent(&self, other: &RichKey) -> bool {
        self.letter_spacing == other.letter_spacing
            && self.word_spacing == other.word_spacing
            && self.kerning == other.kerning
            && self.features.is(&other.features)
            && self.decoration == other.decoration
            && self.overflow == other.overflow
            && self.max_lines == other.max_lines
            && self.capsize == other.capsize
            && self.transform == other.transform
            && self.spans.is(&other.spans)
    }
}

/// A style's shaping features, borrowed for a lookup; it hashes and
/// compares as the `Vec<FontFeature>` that [`RichKey`] keeps.
#[derive(Clone, Copy)]
struct FeaturesRef<'a>(&'a ComputedTextStyle);

impl Hash for FeaturesRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0.features.len() + usize::from(self.0.tabular_nums)).hash(state);
        for feature in self.iter() {
            feature.hash(state);
        }
    }
}

impl FeaturesRef<'_> {
    /// The style's shaping features, in order.
    fn iter(&self) -> impl Iterator<Item = FontFeature> + '_ {
        self.0
            .features
            .iter()
            .copied()
            .chain(self.0.tabular_nums.then(|| FontFeature::new(*b"tnum", 1)))
    }

    /// Whether `features` are this style's shaping features.
    fn is(&self, features: &[FontFeature]) -> bool {
        features.iter().copied().eq(self.iter())
    }

    /// The same features, owned.
    fn to_owned(self) -> Vec<FontFeature> {
        self.iter().collect()
    }
}

/// Every span of a paragraph, borrowed for a lookup; it hashes and compares
/// as the `Vec<SpanBits>` that [`RichKey`] keeps.
#[derive(Clone, Copy)]
struct SpansRef<'a> {
    spans: &'a [TextSpan],
    drawn: bool,
}

impl Hash for SpansRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.spans.len().hash(state);
        for span in self.spans {
            span_bits_ref(span, self.drawn).hash(state);
        }
    }
}

impl SpansRef<'_> {
    /// Whether `bits` are these spans' bits.
    fn is(&self, bits: &[SpanBits]) -> bool {
        bits.len() == self.spans.len()
            && self
                .spans
                .iter()
                .zip(bits)
                .all(|(span, bits)| span_bits_ref(span, self.drawn).equivalent(bits))
    }

    /// The same spans' bits, owned.
    fn to_owned(self) -> Vec<SpanBits> {
        self.spans
            .iter()
            .map(|span| span_bits(span, self.drawn))
            .collect()
    }
}

/// One span's bits, borrowing the span's text; it hashes and compares as
/// the [`SpanBits`] the cache keeps.
fn span_bits_ref(span: &TextSpan, drawn: bool) -> SpanBitsRef<'_> {
    let s = &span.style;
    SpanBitsRef {
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
        family: s.family.as_deref(),
        italic: s.italic,
        letter_spacing: s.letter_spacing.map(f64::to_bits),
        word_spacing: s.word_spacing.map(f64::to_bits),
        kerning: s.kerning,
        features: s.features.as_deref(),
        variations: s.variations.as_deref().map(VariationsRef),
        font_width: s.font_width.map(f64::to_bits),
        decoration: if drawn {
            s.decoration.map(decoration_bits)
        } else {
            None
        },
        highlight: drawn && s.highlight,
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
