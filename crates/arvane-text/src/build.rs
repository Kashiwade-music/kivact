//! Role: Construction of a parley layout from a paragraph and its spans.
//!
//! Building a parley layout from a paragraph: its style as the defaults,
//! each span's shaping style over its range, the span's index (plus one)
//! as the brush, so drawing finds a glyph's span without shaping again
//! (brush 0 is the paragraph's color, decoration and band). Letter and
//! word spacing are parley's: added to each cluster's (each space's)
//! advance.

use kivact_types::{FontFeature, FontKerning, TextSpan, TextStyle, TextWrap};
use parley::{
    FontContext, FontFamily, FontFamilyName, FontStyle, FontWeight, GenericFamily, LayoutContext,
    LineHeight, StyleProperty,
};

/// Shapes `text` (not empty, converted already) with `spans` (valid,
/// moved onto the converted text) in `style` into a parley layout, not
/// broken into lines yet; lines are `height` apart (a span with its own
/// font size sets its own line height: `line_height` of its size).
pub(crate) fn shape_layout(
    fonts: &mut FontContext,
    contexts: &mut LayoutContext<u32>,
    text: &str,
    spans: &[TextSpan],
    style: &TextStyle,
    height: f32,
) -> parley::Layout<u32> {
    let mut builder = contexts.ranged_builder(fonts, text, 1.0, true);
    builder.push_default(StyleProperty::FontSize(style.font_size));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(f32::from(
        style.weight,
    ))));
    if style.italic {
        builder.push_default(StyleProperty::FontStyle(FontStyle::Italic));
    }
    if style.wrap == TextWrap::NoWrap {
        builder.push_default(StyleProperty::TextWrapMode(parley::TextWrapMode::NoWrap));
    }
    if style.letter_spacing != 0.0 {
        builder.push_default(StyleProperty::LetterSpacing(style.letter_spacing));
    }
    if style.word_spacing != 0.0 {
        builder.push_default(StyleProperty::WordSpacing(style.word_spacing));
    }
    let features = settings(style.kerning, &style.features);
    if !features.is_empty() {
        builder.push_default(StyleProperty::FontFeatures(features.as_slice().into()));
    }
    let emoji = text.chars().any(may_be_emoji);
    builder.push_default(StyleProperty::FontFamily(families(
        style.family.as_deref(),
        emoji,
    )));
    builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(height)));
    for (i, span) in spans.iter().enumerate() {
        let range = span.range.clone();
        let s = &span.style;
        builder.push(StyleProperty::Brush(i as u32 + 1), range.clone());
        if let Some(size) = s.font_size {
            builder.push(StyleProperty::FontSize(size), range.clone());
            let own = TextStyle {
                font_size: size,
                ..TextStyle::default()
            };
            let lines = crate::metrics::line_height(
                &TextStyle {
                    line_height: style.line_height,
                    ..own
                },
                None,
            )
            .unwrap_or(height);
            builder.push(
                StyleProperty::LineHeight(LineHeight::Absolute(lines)),
                range.clone(),
            );
        }
        if let Some(weight) = s.weight {
            builder.push(
                StyleProperty::FontWeight(FontWeight::new(f32::from(weight))),
                range.clone(),
            );
        }
        if let Some(italic) = s.italic {
            let face = if italic {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            };
            builder.push(StyleProperty::FontStyle(face), range.clone());
        }
        if let Some(spacing) = s.letter_spacing {
            builder.push(StyleProperty::LetterSpacing(spacing), range.clone());
        }
        if let Some(spacing) = s.word_spacing {
            builder.push(StyleProperty::WordSpacing(spacing), range.clone());
        }
        if s.features.is_some() || s.kerning.is_some() {
            // The span's features replace the paragraph's, kerning with
            // them: whichever it leaves unset is the paragraph's.
            let features = settings(
                s.kerning.unwrap_or(style.kerning),
                s.features.as_deref().unwrap_or(&style.features),
            );
            builder.push(
                StyleProperty::FontFeatures(features.as_slice().into()),
                range.clone(),
            );
        }
        if let Some(family) = &s.family {
            builder.push(
                StyleProperty::FontFamily(families(Some(family), emoji)),
                range,
            );
        }
    }
    builder.build(text)
}

/// Whether `spans` may shape `text`: sorted, not overlapping, within the
/// text and on character boundaries.
pub(crate) fn valid_spans(text: &str, spans: &[TextSpan]) -> bool {
    let mut end = 0;
    spans.iter().all(|span| {
        let ok = span.range.start >= end
            && span.range.start <= span.range.end
            && text.is_char_boundary(span.range.start)
            && text.is_char_boundary(span.range.end);
        end = span.range.end;
        ok
    })
}

/// The features a run is shaped with: its kerning as the `kern` feature
/// first, so a `kern` in `features` (CSS `font-feature-settings` wins
/// over `font-kerning`) comes after it.
fn settings(kerning: FontKerning, features: &[FontFeature]) -> Vec<parley::FontFeature> {
    let kern = match kerning {
        FontKerning::Auto => None,
        FontKerning::Normal => Some(FontFeature::new(*b"kern", 1)),
        FontKerning::None => Some(FontFeature::new(*b"kern", 0)),
    };
    kern.iter()
        .chain(features)
        .map(|f| parley::FontFeature::new(parley::setting::Tag::new(&f.tag), f.value))
        .collect()
}

/// The fonts a paragraph is shaped with: the requested families, then the
/// platform UI font (Segoe UI on Windows, like native controls). Script
/// fallback does not cover common symbols (U+25BE in select buttons), so
/// platform symbol fonts come last; absent names are skipped. Before them,
/// in a text that may hold emoji, the platform's color emoji font: symbol
/// fonts draw many emoji in one color, and parley falls back to the emoji
/// font only after the whole list. (Only such texts: every family in the
/// list costs each cluster the list does not cover, like CJK ones, a
/// lookup.)
fn families(family: Option<&str>, emoji: bool) -> FontFamily<'_> {
    // Preserve the cheap, literal single-name path. Explicit CSS lists
    // choose a per-glyph fallback before the platform UI font, which can
    // otherwise take CJK glyphs from a different installed family.
    let mut families: Vec<FontFamilyName<'_>> = match family {
        Some(source) if source.contains(',') || source.trim_start().starts_with(['\'', '"']) => {
            FontFamilyName::parse_css_list(source)
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|_| vec![FontFamilyName::named(source)])
        }
        _ => family.into_iter().map(FontFamilyName::named).collect(),
    };
    families.push(FontFamilyName::Generic(GenericFamily::SystemUi));
    families.push(FontFamilyName::Generic(GenericFamily::SansSerif));
    if emoji {
        families.push(FontFamilyName::Generic(GenericFamily::Emoji));
    }
    families.extend(
        SYMBOL_FAMILIES
            .iter()
            .map(|name| FontFamilyName::named(name)),
    );
    FontFamily::List(families.into())
}

/// Whether `c` may start or shape an emoji (pictographs, dingbats,
/// arrows and geometric shapes with emoji forms, the emoji variation
/// selector): a coarse superset of the characters with emoji
/// presentation.
fn may_be_emoji(c: char) -> bool {
    matches!(
        u32::from(c),
        0xA9 | 0xAE
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x2194..=0x21FF
            | 0x2300..=0x23FF
            | 0x24C2
            | 0x25A0..=0x27BF
            | 0x2934..=0x2935
            | 0x2B00..=0x2BFF
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0xFE0F
            | 0x1F000..=0x1FAFF
    )
}

/// Symbol fonts shipped with Windows, macOS and common Linux distributions.
const SYMBOL_FAMILIES: [&str; 4] = [
    "Segoe UI Symbol",
    "Apple Symbols",
    "Noto Sans Symbols 2",
    "DejaVu Sans",
];
