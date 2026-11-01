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

use ui_types::{FontFeature, FontKerning, SpanStyle, TextSpan, TextStyle, TextTransform, TextWrap};

/// Layouts per generation.
const ENTRIES: usize = 256;

/// What a span sets that shaping reads; floats compare by bit pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SpanKey {
    start: usize,
    end: usize,
    font_size: Option<u32>,
    weight: Option<u16>,
    family: Option<String>,
    italic: Option<bool>,
    letter_spacing: Option<u32>,
    word_spacing: Option<u32>,
    kerning: Option<FontKerning>,
    features: Option<Vec<FontFeature>>,
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
            ..
        } = &span.style;
        Self {
            start: span.range.start,
            end: span.range.end,
            font_size: font_size.map(f32::to_bits),
            weight: *weight,
            family: family.clone(),
            italic: *italic,
            letter_spacing: letter_spacing.map(f32::to_bits),
            word_spacing: word_spacing.map(f32::to_bits),
            kerning: *kerning,
            features: features.clone(),
        }
    }
}

/// What shaping reads besides the common style, kept out of line: plain
/// text in a plain style keys none (as small and as quick to hash as
/// before).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RichKey {
    letter_spacing: u32,
    word_spacing: u32,
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
    font_size: u32,
    weight: u16,
    line_height: u32,
    wrap: bool,
    family: Option<String>,
    italic: bool,
    rich: Option<Box<RichKey>>,
}

impl Key {
    /// The key of `text` with `spans` in `style` with lines `line_height`
    /// apart (the text before its case conversion).
    pub(crate) fn new(text: &str, spans: &[TextSpan], style: &TextStyle, line_height: f32) -> Self {
        let plain = spans.is_empty()
            && style.letter_spacing == 0.0
            && style.word_spacing == 0.0
            && style.kerning == FontKerning::Auto
            && style.features.is_empty()
            && style.transform == TextTransform::None;
        Self {
            text: text.to_owned(),
            font_size: style.font_size.to_bits(),
            weight: style.weight,
            line_height: line_height.to_bits(),
            wrap: style.wrap == TextWrap::Wrap,
            family: style.family.clone(),
            italic: style.italic,
            rich: (!plain).then(|| {
                Box::new(RichKey {
                    letter_spacing: style.letter_spacing.to_bits(),
                    word_spacing: style.word_spacing.to_bits(),
                    kerning: style.kerning,
                    features: style.features.clone(),
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

    /// Forget every layout (fonts changed).
    pub(crate) fn clear(&mut self) {
        self.current.clear();
        self.previous.clear();
    }
}
