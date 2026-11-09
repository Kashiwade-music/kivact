//! Role: Cache of shaped paragraphs kept across calls and reused at every width.
//!
//! Shaped paragraphs kept across calls: shaping (font selection, bidi,
//! glyphs, break opportunities) is most of a paragraph's cost, and its
//! result serves every width the paragraph is measured or drawn at, as
//! breaking lines again is cheap.
//!
//! Bounded: two generations of [`ENTRIES`] layouts; an entry used from the
//! older generation moves to the newer one, and a full newer generation
//! drops the older one.
//!
//! Invariant: a key holds what shaping reads and nothing else: the
//! colors, decorations, highlights, alignment, overflow and line clamp of
//! a paragraph and its spans are applied to the shaped layout afterwards,
//! so changing them never shapes again. Plain text in a plain style keys
//! nothing but the text and its common style (no allocation besides the
//! text, as quick to hash as before the rest of the vocabulary).

use std::collections::HashMap;

use kivact_types::render::SpanStyle;
use kivact_types::render::TextSpan;
use kivact_types::{ComputedTextStyle, FontFeature, FontVariation, TextWrap};
use kivact_types::{FontKerning, TextTransform};

/// Layouts per generation.
const ENTRIES: usize = 256;

/// A variable-font axis setting as bits: its tag and its value's bit
/// pattern.
pub(crate) type VariationBits = ([u8; 4], u64);

/// The axis settings `variations` as bits, in order.
pub(crate) fn variation_bits(variations: &[FontVariation]) -> Vec<VariationBits> {
    variations
        .iter()
        .map(|v| (v.tag, v.value.to_bits()))
        .collect()
}

/// What a span sets that shaping reads; floats compare by bit pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SpanKey {
    start: usize,
    end: usize,
    // xtask-allow(state-fields): cache key - font size
    font_size: Option<u64>,
    // xtask-allow(state-fields): cache key - weight
    weight: Option<u16>,
    // xtask-allow(state-fields): cache key - family
    family: Option<String>,
    // xtask-allow(state-fields): cache key - italic
    italic: Option<bool>,
    // xtask-allow(state-fields): cache key - letter spacing
    letter_spacing: Option<u64>,
    // xtask-allow(state-fields): cache key - word spacing
    word_spacing: Option<u64>,
    // xtask-allow(state-fields): cache key - kerning
    kerning: Option<FontKerning>,
    // xtask-allow(state-fields): cache key - features
    features: Option<Vec<FontFeature>>,
    // xtask-allow(state-fields): cache key - variations
    variations: Option<Vec<VariationBits>>,
    // xtask-allow(state-fields): cache key - font width
    font_width: Option<u64>,
}

impl SpanKey {
    fn new(span: &TextSpan) -> Self {
        let SpanStyle {
            font_size,
            weight,
            family,
            italic,
            letter_spacing,
            word_spacing,
            kerning,
            features,
            variations,
            font_width,
            ..
        } = &span.style;
        Self {
            start: span.range.start,
            end: span.range.end,
            font_size: font_size.map(f64::to_bits),
            weight: *weight,
            family: family.clone(),
            italic: *italic,
            letter_spacing: letter_spacing.map(f64::to_bits),
            word_spacing: word_spacing.map(f64::to_bits),
            kerning: *kerning,
            features: features.clone(),
            variations: variations.as_deref().map(variation_bits),
            font_width: font_width.map(f64::to_bits),
        }
    }
}

/// What shaping reads besides the common style, kept out of line: plain
/// text in a plain style keys none (as small and as quick to hash as
/// before).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RichKey {
    letter_spacing: u64,
    word_spacing: u64,
    kerning: FontKerning,
    features: Vec<FontFeature>,
    transform: TextTransform,
    spans: Vec<SpanKey>,
}

/// What a shaped layout depends on: the text, the paragraph style that
/// shaping reads (see the module invariant) and its spans' ranges and
/// shaping styles; floats compare by bit pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    text: String,
    font_size: u64,
    weight: u16,
    line_height: u64,
    // xtask-allow(state-fields): cache key - wrap
    wrap: bool,
    // xtask-allow(state-fields): cache key - family
    family: Option<String>,
    // xtask-allow(state-fields): cache key - italic
    italic: bool,
    variations: Vec<VariationBits>,
    font_width: u64,
    // xtask-allow(state-fields): cache key - optical sizing
    optical_sizing: bool,
    // xtask-allow(state-fields): cache key - rich
    rich: Option<Box<RichKey>>,
}

impl Key {
    /// The key of `text` with `spans` in `style` with lines `line_height`
    /// apart (the text before its case conversion).
    pub(crate) fn new(
        text: &str,
        spans: &[TextSpan],
        style: &ComputedTextStyle,
        line_height: f64,
    ) -> Self {
        let plain = spans.is_empty()
            && style.resolve(style.letter_spacing) == 0.0
            && style.resolve(style.word_spacing) == 0.0
            && style.kerning == FontKerning::Auto
            && style.shaping_features().is_empty()
            && style.transform == TextTransform::None;
        Self {
            text: text.to_owned(),
            font_size: style.font_size.to_bits(),
            weight: style.weight,
            line_height: line_height.to_bits(),
            wrap: style.wrap == TextWrap::Wrap,
            family: style.family.clone(),
            italic: style.italic,
            variations: variation_bits(&style.variations),
            font_width: style.font_width.to_bits(),
            optical_sizing: style.optical_sizing,
            rich: (!plain).then(|| {
                Box::new(RichKey {
                    letter_spacing: style.resolve(style.letter_spacing).to_bits(),
                    word_spacing: style.resolve(style.word_spacing).to_bits(),
                    kerning: style.kerning,
                    features: style.shaping_features(),
                    transform: style.transform,
                    spans: spans.iter().map(SpanKey::new).collect(),
                })
            }),
        }
    }
}

/// Shaped layouts by [`Key`]; a glyph's brush is the index of its span
/// plus one (0: the paragraph's style).
#[derive(Default)]
pub(crate) struct Shaped {
    current: HashMap<Key, parley::Layout<u32>>,
    previous: HashMap<Key, parley::Layout<u32>>,
}

impl Shaped {
    /// The layout of `key`, shaped by `shape` unless kept.
    pub(crate) fn get_or_insert(
        &mut self,
        key: Key,
        shape: impl FnOnce() -> parley::Layout<u32>,
    ) -> &mut parley::Layout<u32> {
        if !self.current.contains_key(&key) {
            let layout = self.previous.remove(&key).unwrap_or_else(shape);
            if self.current.len() >= ENTRIES {
                self.previous = std::mem::take(&mut self.current);
            }
            self.current.insert(key.clone(), layout);
        }
        self.current
            .get_mut(&key)
            .expect("invariant: inserted above")
    }
}
