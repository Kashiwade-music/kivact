//! Role: TextEngine: font registration, and cached measuring and shaping of text.

use crate::cache::{Cache, TextKeyRef};
use crate::error::TextError;
use crate::metrics::line_height;
use crate::{build, layouts, output, transform};
use kivact_types::render::{RichText, ShapedText};
use kivact_types::{ComputedTextStyle, TextDecoration, TextWrap};
use kivact_types::{Rect, TextAlign, TextCaret, TextMetrics, TextOverflow, TextTransform};
use parley::{
    Alignment, AlignmentOptions, FontContext, GenericFamily, LayoutContext, PositionedLayoutItem,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Reusable font database and shaping scratch space, used on its owner's UI thread.
///
/// Each application owns an engine; there is no mutable global font state.
#[derive(Default)]
pub struct TextEngine {
    pub(super) cache: Cache,
    pub(crate) fonts: FontContext,
    pub(crate) layouts: LayoutContext<u32>,
    pub(super) font_bytes: HashMap<u64, Arc<[u8]>>,
}

/// Ellipses kept before they are all forgotten.
const ELLIPSES: usize = 64;

impl TextEngine {
    /// Creates an engine which discovers fonts installed on the system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an engine that never sees system fonts: only `fonts` exist,
    /// and they become every generic family (the system UI family `shape`
    /// falls back to included), so shaping and measurement are the same on
    /// every machine. Text none of them covers draws the missing glyph of
    /// the first font. Fonts registered later join the collection as usual.
    pub fn isolated(fonts: &[Arc<[u8]>]) -> Result<Self, TextError> {
        use parley::fontique::{Collection, CollectionOptions, SourceCache};
        let mut engine = Self {
            cache: Cache::default(),
            fonts: FontContext {
                collection: Collection::new(CollectionOptions {
                    shared: false,
                    system_fonts: false,
                }),
                source_cache: SourceCache::default(),
            },
            layouts: LayoutContext::new(),
            font_bytes: HashMap::new(),
        };
        let mut families = Vec::new();
        for bytes in fonts {
            let faces = engine
                .fonts
                .collection
                .register_fonts(parley::fontique::Blob::new(Arc::new(bytes.clone())), None);
            if faces.is_empty() {
                return Err(TextError::InvalidFont);
            }
            for (family, _) in faces {
                if !families.contains(&family) {
                    families.push(family);
                }
            }
        }
        for generic in [
            GenericFamily::SystemUi,
            GenericFamily::UiSansSerif,
            GenericFamily::SansSerif,
            GenericFamily::Serif,
            GenericFamily::Monospace,
        ] {
            engine
                .fonts
                .collection
                .set_generic_families(generic, families.iter().copied());
        }
        Ok(engine)
    }

    /// Registers owned font bytes and returns the discovered family names.
    ///
    /// Registration allows applications and headless tests to choose explicit fonts.
    pub fn register_font(&mut self, bytes: Arc<[u8]>) -> Result<Vec<String>, TextError> {
        let faces = self
            .fonts
            .collection
            .register_fonts(parley::fontique::Blob::new(Arc::new(bytes)), None);
        if faces.is_empty() {
            return Err(TextError::InvalidFont);
        }
        // A successful registration can change every answer computed from the fonts.
        self.cache = Cache::default();
        Ok(faces
            .into_iter()
            .filter_map(|(id, _)| self.fonts.collection.family_name(id).map(str::to_owned))
            .collect())
    }

    /// Registered and system font family names, sorted and unique, preserving case.
    /// Enumerated lazily and retained across cache release until successful registration.
    pub fn family_names(&mut self) -> Vec<String> {
        if let Some(names) = &self.cache.font_names {
            return names.clone();
        }
        let mut names: Vec<_> = self
            .fonts
            .collection
            .family_names()
            .map(str::to_owned)
            .collect();
        names.sort();
        names.dedup();
        self.cache.font_names = Some(names.clone());
        names
    }

    /// Whether a registered or system font has the family `name`, ignoring
    /// case. Answers remain cached until registration or cache release.
    pub fn has_family(&mut self, name: &str) -> bool {
        let key = name.to_lowercase();
        if let Some(found) = self.cache.families.get(&key) {
            return *found;
        }
        let found = self.fonts.collection.family_id(name).is_some();
        self.cache.families.insert(key, found);
        found
    }

    /// Release cached layouts, ellipses, copied font bytes, loaded system
    /// font sources and shaping scratch when memory is short.
    ///
    /// Font discovery, registered bytes and the family-name snapshot remain.
    /// Returned glyph runs own their font copies independently and remain
    /// valid; later calls shape again and reload system sources as needed.
    pub fn release_caches(&mut self) {
        let font_names = self.cache.font_names.take();
        self.cache = Cache {
            font_names,
            ..Cache::default()
        };
        self.font_bytes = HashMap::new();
        self.fonts.source_cache.prune(0, true);
        self.layouts = LayoutContext::new();
    }

    /// Measure a paragraph, caching successes and errors by text, style and width.
    /// Drawing-only properties are excluded from the measurement key.
    pub fn measure(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
    ) -> Result<TextMetrics, TextError> {
        let key = TextKeyRef::new(text, style, max_width, false);
        if let Some(result) = self.cache.measured.get(&key) {
            return result;
        }
        let result = self.measure_uncached(text, style, max_width);
        self.cache.measured.insert(key.to_owned(), result.clone());
        result
    }

    /// Measures the extent `shape` would produce without building glyph runs or carets.
    ///
    /// A paragraph is shaped once per text and style (color and alignment
    /// aside): measuring or shaping it again at another width only breaks
    /// its lines again.
    fn measure_uncached(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
    ) -> Result<TextMetrics, TextError> {
        let height = line_height(style, max_width)?;
        if text.text.is_empty() {
            return Ok(TextMetrics::default());
        }
        if !build::valid_spans(text.text, text.spans) {
            return Err(TextError::InvalidSpans);
        }
        let ellipsis = self.ellipsis_for(text, style, max_width, height);
        let layout = layout_at(
            &mut self.cache.paragraphs,
            &mut self.fonts,
            &mut self.layouts,
            text,
            style,
            max_width,
            height,
        );
        let shaped = layout.lines().any(|line| {
            line.items()
                .any(|item| matches!(item, PositionedLayoutItem::GlyphRun(_)))
        });
        if !shaped
            && text
                .text
                .chars()
                .any(|character| !character.is_whitespace())
        {
            return Err(TextError::MissingFont);
        }
        let plan = output::plan(layout, style, max_width, ellipsis.as_ref());
        let mut metrics = TextMetrics {
            size: plan.size,
            baseline: plan.baseline,
        };
        if let Some((top, bottom)) = plan.cap_edges(layout, style) {
            metrics.size.height = bottom - top;
            metrics.baseline -= top;
        }
        Ok(metrics)
    }

    /// Shapes, wraps, and aligns text in logical pixels using actual font glyphs.
    /// Successes and errors are cached; repeated calls share the same `Arc`.
    /// Cache keys include drawing properties, width and whether carets are needed.
    pub fn shape(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
    ) -> Result<kivact_types::ShapedTextSnapshot, TextError> {
        self.shape_cached(text, style, max_width, true)
    }

    /// [`TextEngine::shape`] for drawing only: the same size and glyph
    /// runs, without carets (`carets` is empty).
    pub fn shape_glyphs(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
    ) -> Result<kivact_types::ShapedTextSnapshot, TextError> {
        self.shape_cached(text, style, max_width, false)
    }

    fn shape_cached(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
        carets: bool,
    ) -> Result<kivact_types::ShapedTextSnapshot, TextError> {
        let key = TextKeyRef::new(text, style, max_width, true).with_carets(carets);
        if let Some(result) = self.cache.shaped.get(&key) {
            return result;
        }
        let result = self
            .shape_with(text, style, max_width, carets)
            .map(kivact_types::ShapedTextSnapshot::new);
        self.cache.shaped.insert(key.to_owned(), result.clone());
        result
    }

    /// The ellipsis a paragraph cuts its lines with, shaped once per
    /// paragraph style; `None` when no line of it is cut.
    fn ellipsis_for(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
        height: f64,
    ) -> Option<output::Ellipsis> {
        if !style.ellipsizes() {
            return None;
        }
        let layout = layout_at(
            &mut self.cache.paragraphs,
            &mut self.fonts,
            &mut self.layouts,
            text,
            style,
            max_width,
            height,
        );
        if !output::needs_cut(layout, style, max_width) {
            return None;
        }
        let own = ComputedTextStyle {
            wrap: TextWrap::NoWrap,
            align: TextAlign::Start,
            overflow: TextOverflow::Clip,
            max_lines: None,
            transform: TextTransform::None,
            decoration: TextDecoration::default(),
            ..style.clone()
        };
        let key = layouts::Key::new(ELLIPSIS, &[], &own, height);
        if let Some(e) = self.cache.ellipses.get(&key) {
            return Some(e.clone());
        }
        let mut layout = build::shape_layout(
            &mut self.fonts,
            &mut self.layouts,
            ELLIPSIS,
            &[],
            &own,
            height,
        );
        layout.break_all_lines(None);
        let ellipsis = output::Ellipsis::of(&layout, &mut self.font_bytes);
        if self.cache.ellipses.len() >= ELLIPSES {
            self.cache.ellipses.clear();
        }
        self.cache.ellipses.insert(key, ellipsis.clone());
        Some(ellipsis)
    }

    fn shape_with(
        &mut self,
        text: RichText<'_>,
        style: &ComputedTextStyle,
        max_width: Option<f64>,
        with_carets: bool,
    ) -> Result<ShapedText, TextError> {
        let height = line_height(style, max_width)?;
        if text.text.is_empty() {
            return Ok(empty_text(style, max_width, height, with_carets));
        }
        if !build::valid_spans(text.text, text.spans) {
            return Err(TextError::InvalidSpans);
        }
        let ellipsis = self.ellipsis_for(text, style, max_width, height);
        let layout = layout_at(
            &mut self.cache.paragraphs,
            &mut self.fonts,
            &mut self.layouts,
            text,
            style,
            max_width,
            height,
        );
        let alignment = match style.align {
            TextAlign::Start => Alignment::Start,
            TextAlign::Center => Alignment::Center,
            TextAlign::End => Alignment::End,
            TextAlign::Justify => Alignment::Justify,
        };
        layout.align(alignment, AlignmentOptions::default());
        let plan = output::plan(layout, style, max_width, ellipsis.as_ref());
        let drawn = output::draw(
            layout,
            style,
            text.spans,
            &plan,
            ellipsis.as_ref(),
            &mut self.font_bytes,
        );
        if drawn.runs.is_empty()
            && text
                .text
                .chars()
                .any(|character| !character.is_whitespace())
        {
            return Err(TextError::MissingFont);
        }
        // Byte offsets of the layout's text (converted) in the caller's.
        let boundaries = transform::boundaries(text.text, style.transform);
        let lines = output::lines(layout, &plan, style.align == TextAlign::Justify, |range| {
            match &boundaries {
                Some(pairs) => {
                    transform::original(pairs, range.start)..transform::original(pairs, range.end)
                }
                None => range,
            }
        });
        let cluster_boundaries = cluster_boundaries(layout, text.text, style);
        let carets = if with_carets {
            // Offsets in the converted text (its case conversion).
            let (converted, _) = transform::apply(text.text, &[], style.transform);
            carets(layout, &crlf_as_one_break(&converted))
        } else {
            Vec::new()
        };
        let mut shaped = ShapedText {
            size: plan.size,
            runs: drawn.runs,
            carets,
            cluster_boundaries,
            baseline: plan.baseline,
            decorations: drawn.decorations,
            highlights: drawn.highlights,
            lines,
        };
        if let Some((top, bottom)) = plan.cap_edges(layout, style) {
            trim_capsize(&mut shaped, top, bottom);
        }
        Ok(shaped)
    }
}

fn trim_capsize(shaped: &mut ShapedText, top: f64, bottom: f64) {
    shaped.size.height = bottom - top;
    shaped.baseline -= top;
    for run in &mut shaped.runs {
        for glyph in &mut run.make_mut().glyphs {
            glyph.y -= top;
        }
    }
    for line in &mut shaped.lines {
        line.baseline -= top;
        line.rect.y -= top;
    }
    for caret in &mut shaped.carets {
        caret.rect.y -= top;
    }
    for decoration in &mut shaped.decorations {
        decoration.rect.y -= top;
    }
    for highlight in &mut shaped.highlights {
        highlight.rect.y -= top;
    }
}

/// The text "…" (U+2026) an ellipsis draws.
const ELLIPSIS: &str = "\u{2026}";

/// Caret rectangles at every scalar boundary of `text` in `layout`.
fn carets(layout: &parley::Layout<u32>, text: &str) -> Vec<TextCaret> {
    text.char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(text.len()))
        .enumerate()
        .map(|(offset, byte)| {
            let bounds =
                parley::Cursor::from_byte_index(layout, byte, parley::Affinity::Downstream)
                    .geometry(layout, 1.0);
            TextCaret {
                offset: offset as u32,
                rect: Rect {
                    x: bounds.x0,
                    y: bounds.y0,
                    width: bounds.x1 - bounds.x0,
                    height: bounds.y1 - bounds.y0,
                },
            }
        })
        .collect()
}

/// The parley layout of `text` (not empty, valid spans) in `style`, shaped
/// once per text and style (`shaped`) and broken into lines at
/// `max_width`.
///
/// A text of white space alone is broken only at its line breaks: its
/// spaces hang past any width, as preserved white space does in CSS.
/// Parley would break a line after every space wider than the width (also
/// without wrapping, for the first space of a line) and end with an empty
/// line, so a blank line measured at a narrow width would be several lines
/// high.
fn layout_at<'a>(
    shaped: &'a mut layouts::Shaped,
    fonts: &mut FontContext,
    contexts: &mut LayoutContext<u32>,
    text: RichText<'_>,
    style: &ComputedTextStyle,
    max_width: Option<f64>,
    height: f64,
) -> &'a mut parley::Layout<u32> {
    let key = layouts::Key::new(text.text, text.spans, style, height);
    let layout = shaped.get_or_insert(key, || {
        let (converted, spans) = transform::apply(text.text, text.spans, style.transform);
        let spans = crlf_spans(&converted, spans);
        let converted = crlf_as_one_break(&converted);
        build::shape_layout(fonts, contexts, &converted, &spans, style, height)
    });
    let blank = text.text.chars().all(char::is_whitespace);
    layout.break_all_lines(if blank {
        None
    } else {
        max_width.map(|value| value as f32)
    });
    layout
}

/// Parley breaks a line at both CR and LF; a CR before LF becomes an
/// invisible zero width space so CRLF breaks once while every scalar keeps
/// its offset (and caret).
pub(super) fn crlf_as_one_break(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.contains("\r\n") {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 2);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(if c == '\r' && chars.peek() == Some(&'\n') {
            '\u{200B}'
        } else {
            c
        });
    }
    std::borrow::Cow::Owned(out)
}

/// Span ranges follow [`crlf_as_one_break`]. A CR before an LF grows from one
/// byte to three there, so every range after it moves by two bytes. Ranges
/// are in `text`, the layout is built on the normalized text.
pub(super) fn crlf_spans<'a>(
    text: &str,
    spans: std::borrow::Cow<'a, [kivact_types::render::TextSpan]>,
) -> std::borrow::Cow<'a, [kivact_types::render::TextSpan]> {
    if spans.is_empty() || !text.contains("\r\n") {
        return spans;
    }
    // Where each CR that precedes an LF ends.
    let crs: Vec<usize> = text
        .match_indices("\r\n")
        .map(|(at, _)| at + '\r'.len_utf8())
        .collect();
    let grown = '\u{200B}'.len_utf8() - '\r'.len_utf8();
    let normalized = |at: usize| at + crs.partition_point(|end| *end <= at) * grown;
    std::borrow::Cow::Owned(
        spans
            .into_owned()
            .into_iter()
            .map(|mut span| {
                span.range = normalized(span.range.start)..normalized(span.range.end);
                span
            })
            .collect(),
    )
}

/// Scalar boundaries at the starts of shaped visual clusters.
fn cluster_boundaries(
    layout: &parley::Layout<u32>,
    text: &str,
    style: &ComputedTextStyle,
) -> Vec<u32> {
    let (converted, _) = transform::apply(text, &[], style.transform);
    let converted = crlf_as_one_break(&converted);
    let scalar_bytes: Vec<usize> = converted
        .char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(converted.len()))
        .collect();
    let mut cluster_boundaries = vec![0, (scalar_bytes.len() - 1) as u32];
    for line in layout.lines() {
        for run in line.runs() {
            for cluster in run.visual_clusters() {
                if !cluster.is_ligature_continuation() {
                    let byte = cluster.text_range().start;
                    cluster_boundaries.push(scalar_bytes.partition_point(|&at| at < byte) as u32);
                }
            }
        }
    }
    cluster_boundaries.sort_unstable();
    cluster_boundaries.dedup();
    cluster_boundaries
}

/// Preserve an empty paragraph's alignment and optional editing caret.
fn empty_text(
    style: &ComputedTextStyle,
    max_width: Option<f64>,
    height: f64,
    with_carets: bool,
) -> ShapedText {
    let x = match style.align {
        TextAlign::Start | TextAlign::Justify => 0.0,
        TextAlign::Center => max_width.unwrap_or(0.0) / 2.0,
        TextAlign::End => max_width.unwrap_or(0.0),
    };
    let caret = TextCaret {
        offset: 0,
        rect: Rect {
            x,
            y: 0.0,
            width: 1.0,
            height,
        },
    };
    ShapedText {
        carets: if with_carets { vec![caret] } else { Vec::new() },
        cluster_boundaries: vec![0],
        ..ShapedText::default()
    }
}
