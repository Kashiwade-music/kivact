//! Role: Cache of shaped paragraphs kept across calls and reused at every width.
//!
//! Shaped paragraphs kept across calls: shaping (font selection, bidi,
//! glyphs, break opportunities) is most of a paragraph's cost, and its
//! result serves every width the paragraph is measured or drawn at, as
//! breaking lines again is cheap.
//!
//! Bounded: two generations of [`ENTRIES`] layouts; an entry used from the
//! older generation moves to the newer one, and a full newer generation
//! drops the older one. A paragraph is kept from its second shaping on (a
//! text measured once and never drawn keeps only its key's hash,
//! `Shaped::seen`).
//!
//! Invariant: a key holds what shaping reads and nothing else: the
//! colors, decorations, highlights, alignment, overflow and line clamp of
//! a paragraph and its spans are applied to the shaped layout afterwards,
//! so changing them never shapes again. Plain text in a plain style keys
//! nothing but the text and its common style (no allocation besides the
//! text, as quick to hash as before the rest of the vocabulary).

use std::collections::{HashMap, HashSet};

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
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    font_size: Option<u64>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    weight: Option<u16>,
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
    variations: Option<Vec<VariationBits>>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
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
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    wrap: bool,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    family: Option<String>,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    italic: bool,
    variations: Vec<VariationBits>,
    font_width: u64,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
    optical_sizing: bool,
    // xtask-allow(state-fields): cache: one component of a cache key; equality of the whole key is what matters
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
///
/// A paragraph is kept from its second shaping on: the first time a key is
/// shaped its layout stays in `scratch` only (until another first-time
/// paragraph replaces it) and its hash is remembered in `seen`, so a text
/// measured once and never drawn keeps nothing but the hash.
#[derive(Default)]
pub(crate) struct Shaped {
    current: HashMap<Key, parley::Layout<u32>>,
    previous: HashMap<Key, parley::Layout<u32>>,
    /// The last paragraph shaped for the first time.
    scratch: Option<(Key, parley::Layout<u32>)>,
    /// The hashes of the keys shaped once and not kept: two generations of
    /// at most `SEEN` each, as the layouts' two generations.
    seen: [HashSet<u64>; 2],
}

/// Whether a layout was kept from an earlier call or shaped by this one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Found {
    Kept,
    Shaped,
}

/// First-time keys remembered per generation.
const SEEN: usize = 8192;

/// The hash `seen` keeps for `key` (one fixed hasher: the maps' own are
/// seeded per map and change when a generation is replaced).
fn key_hash(key: &Key) -> u64 {
    use std::hash::BuildHasher;
    std::hash::BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default()
        .hash_one(key)
}

impl Shaped {
    /// The layout of `key`: kept, the scratch's, or shaped by `shape` (kept
    /// when the key was shaped once before, else into the scratch).
    pub(crate) fn get_or_insert(
        &mut self,
        key: Key,
        shape: impl FnOnce() -> parley::Layout<u32>,
    ) -> (&mut parley::Layout<u32>, Found) {
        if self.current.contains_key(&key) {
            let layout = self.current.get_mut(&key).expect("invariant: present");
            return (layout, Found::Kept);
        }
        if let Some(layout) = self.previous.remove(&key) {
            return (self.keep(key, layout), Found::Kept);
        }
        if self.scratch.as_ref().is_some_and(|(kept, _)| *kept == key) {
            let (_, layout) = self.scratch.as_mut().expect("invariant: matched");
            return (layout, Found::Kept);
        }
        let hash = key_hash(&key);
        let layout = shape();
        if self.seen[0].remove(&hash) || self.seen[1].remove(&hash) {
            return (self.keep(key, layout), Found::Shaped);
        }
        if self.seen[0].len() >= SEEN {
            self.seen[1] = std::mem::take(&mut self.seen[0]);
        }
        self.seen[0].insert(hash);
        let (_, layout) = self.scratch.insert((key, layout));
        (layout, Found::Shaped)
    }

    /// Keep `layout` in the current generation (a full one becomes the previous).
    fn keep(&mut self, key: Key, layout: parley::Layout<u32>) -> &mut parley::Layout<u32> {
        if self.current.len() >= ENTRIES {
            self.previous = std::mem::take(&mut self.current);
        }
        self.current.entry(key).or_insert(layout)
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for what the paragraph and measurement caches keep, and when.
    use super::*;
    use crate::TextEngine;
    use crate::cache::TextKeyRef;
    use kivact_types::render::RichText;
    use std::sync::Arc;

    /// The bundled test font (Tuffy; its file lives in the `arvane` crate).
    static FONT: &[u8] = include_bytes!("../../arvane/assets/fonts/Tuffy.ttf");

    fn engine() -> TextEngine {
        TextEngine::isolated(&[Arc::from(FONT)]).expect("the test font loads")
    }

    /// Whether `measured` keeps the answer for `text` in `style`.
    fn measured_kept(engine: &mut TextEngine, text: &str, style: &ComputedTextStyle) -> bool {
        let key = TextKeyRef::new(RichText::from(text), style, None, false);
        engine.cache.measured.get(&key).is_some()
    }

    #[test]
    fn texts_measured_once_each_keep_no_paragraph() {
        let mut engine = engine();
        let style = ComputedTextStyle::default();
        engine
            .measure("first".into(), &style, None)
            .expect("measures");
        engine
            .measure("second".into(), &style, None)
            .expect("measures");
        assert_eq!(engine.cache.paragraphs.current.len(), 0);
    }

    #[test]
    fn a_paragraph_is_kept_from_its_second_shaping() {
        let mut engine = engine();
        let style = ComputedTextStyle::default();
        engine
            .measure("again".into(), &style, None)
            .expect("measures");
        engine
            .measure("other".into(), &style, None)
            .expect("measures");
        engine
            .measure("again".into(), &style, None)
            .expect("measures");
        assert_eq!(engine.cache.paragraphs.current.len(), 1);
    }

    #[test]
    fn a_measurement_is_kept_from_its_third_call_with_equal_metrics() {
        let mut engine = engine();
        let style = ComputedTextStyle::default();
        let first = engine.measure("again".into(), &style, None);
        engine
            .measure("other".into(), &style, None)
            .expect("measures");
        let second = engine.measure("again".into(), &style, None);
        assert!(!measured_kept(&mut engine, "again", &style));
        let third = engine.measure("again".into(), &style, None);
        assert!(measured_kept(&mut engine, "again", &style));
        assert!(!measured_kept(&mut engine, "other", &style));
        assert_eq!(first, second);
        assert_eq!(second, third);
    }
}
