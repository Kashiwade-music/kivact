//! Role: Bounded text snapshots and font queries owned by one text engine.
//! Snapshot keys preserve text, style, width and caret distinctions.

use kivact_types::protocol::{FontFeature, RichText, TextDecoration, TextStyle, TextWrap};
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

    pub(crate) fn clear(&mut self) {
        self.current.clear();
        self.previous.clear();
    }
}

fn color_bits(c: Rgba) -> [u32; 4] {
    [c.r, c.g, c.b, c.a].map(f32::to_bits)
}

/// A decoration's lines, color, thickness and offset, as bits.
type DecorationBits = (u8, Option<[u32; 4]>, Option<u32>, u32);

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
        d.color.map(color_bits),
        d.thickness.map(f32::to_bits),
        d.underline_offset.to_bits(),
    )
}

/// What a span sets, as bits; what drawing alone reads (its color,
/// decoration and highlight) is left out for measurement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SpanBits {
    start: usize,
    end: usize,
    font_size: Option<u32>,
    weight: Option<u16>,
    color: Option<[u32; 4]>,
    family: Option<String>,
    italic: Option<bool>,
    letter_spacing: Option<u32>,
    word_spacing: Option<u32>,
    kerning: Option<FontKerning>,
    features: Option<Vec<FontFeature>>,
    decoration: Option<DecorationBits>,
    highlight: bool,
}

/// The rest of the text vocabulary in a key, kept out of line: plain text
/// in a plain style keys none (its key is as small and as quick to hash
/// as before the vocabulary existed).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RichKey {
    letter_spacing: u32,
    word_spacing: u32,
    kerning: FontKerning,
    features: Vec<FontFeature>,
    decoration: Option<DecorationBits>,
    overflow: bool,
    max_lines: Option<u32>,
    transform: TextTransform,
    spans: Vec<SpanBits>,
}

/// Exact text, spans, style and wrap width; floats compare by bit
/// pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TextKey {
    text: String,
    font_size: u32,
    weight: u16,
    /// Absent for measurement, which color cannot change.
    color: Option<[u32; 4]>,
    line_height: (u8, u32),
    wrap: bool,
    align: u8,
    family: Option<String>,
    italic: bool,
    max_width: Option<u32>,
    /// Whether the shaped text has its carets.
    carets: bool,
    rich: Option<Box<RichKey>>,
}

impl TextKey {
    /// The key of `text` in `style` at `max_width`; `drawn` keys what only
    /// drawing reads too (colors, decorations, highlights).
    pub(crate) fn new(
        text: RichText<'_>,
        style: &TextStyle,
        max_width: Option<f32>,
        drawn: bool,
    ) -> Self {
        let decoration = (drawn && style.decoration != TextDecoration::default())
            .then(|| decoration_bits(style.decoration));
        let plain = text.spans.is_empty()
            && style.letter_spacing == 0.0
            && style.word_spacing == 0.0
            && style.kerning == FontKerning::Auto
            && style.features.is_empty()
            && decoration.is_none()
            && style.overflow == TextOverflow::Clip
            && style.max_lines.is_none()
            && style.transform == TextTransform::None;
        let rich = (!plain).then(|| {
            Box::new(RichKey {
                letter_spacing: style.letter_spacing.to_bits(),
                word_spacing: style.word_spacing.to_bits(),
                kerning: style.kerning,
                features: style.features.clone(),
                decoration,
                overflow: style.overflow == TextOverflow::Ellipsis,
                max_lines: style.max_lines,
                transform: style.transform,
                spans: text
                    .spans
                    .iter()
                    .map(|span| {
                        let s = &span.style;
                        SpanBits {
                            start: span.range.start,
                            end: span.range.end,
                            font_size: s.font_size.map(f32::to_bits),
                            weight: s.weight,
                            color: if drawn { s.color.map(color_bits) } else { None },
                            family: s.family.clone(),
                            italic: s.italic,
                            letter_spacing: s.letter_spacing.map(f32::to_bits),
                            word_spacing: s.word_spacing.map(f32::to_bits),
                            kerning: s.kerning,
                            features: s.features.clone(),
                            decoration: if drawn {
                                s.decoration.map(decoration_bits)
                            } else {
                                None
                            },
                            highlight: drawn && s.highlight,
                        }
                    })
                    .collect(),
            })
        });
        Self {
            text: text.text.to_owned(),
            font_size: style.font_size.to_bits(),
            weight: style.weight,
            color: drawn.then(|| color_bits(style.color)),
            line_height: match style.line_height {
                LineHeight::Normal => (0, 0),
                LineHeight::Scale(v) => (1, v.to_bits()),
                LineHeight::Px(v) => (2, v.to_bits()),
            },
            wrap: style.wrap == TextWrap::Wrap,
            align: match style.align {
                TextAlign::Start => 0,
                TextAlign::Center => 1,
                TextAlign::End => 2,
                TextAlign::Justify => 3,
            },
            family: style.family.clone(),
            italic: style.italic,
            max_width: max_width.map(f32::to_bits),
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

/// Shaped texts kept per cache generation.
const SHAPED_ENTRIES: usize = 1024;
/// Measured extents kept per cache generation.
const MEASURED_ENTRIES: usize = 4096;

/// All snapshots and font queries owned by one text engine.
pub(crate) struct Cache {
    pub(crate) shaped: Generations<
        TextKey,
        Result<std::sync::Arc<kivact_types::protocol::ShapedText>, crate::TextError>,
    >,
    pub(crate) measured: Generations<TextKey, Result<kivact_types::TextMetrics, crate::TextError>>,
    pub(crate) families: HashMap<String, bool>,
    pub(crate) font_names: Option<Vec<String>>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            shaped: Generations::new(SHAPED_ENTRIES),
            measured: Generations::new(MEASURED_ENTRIES),
            families: HashMap::new(),
            font_names: None,
        }
    }
}
