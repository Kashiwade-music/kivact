//! Role: Semantic conversion of wire values to neutral text and geometry.
//!
//! Shared kivact-types values pass through unchanged. Only contracts whose
//! representations differ are resolved here: safe areas, text lengths,
//! shaping styles and spans. Tree layout and input admission stay with callers.
//!
//! Invariants:
//! - The checks of these values stay on the host side of the boundary: a
//!   line height must be positive and finite (else `Normal`; Arvane gets
//!   one of its three values), colors pass through `sanitize::color`,
//!   a safe area's negative or non-finite side
//!   is zero, a non-positive font size is Arvane's default.
//! - Text lengths in `em` are resolved here against the font size (each
//!   span's own): letter and word spacing (Arvane adds word spacing after
//!   every space), a decoration's thickness and underline offset; a
//!   non-finite length is zero and a thickness that is not positive is the
//!   font's. Font features arrive validated by kivact (its `FontFeature`
//!   drops bad tags) and pass with the kerning as they are; Arvane puts the
//!   kerning's `kern` first and an explicit `kern` feature wins.
//! - A rich text span keeps only what it sets: its em lengths resolved
//!   against its own size, its colors drawn through the node's tint, its
//!   family resolved like a text's, and a background as a highlight band.
use crate::sanitize::color;
use kivact_wire as wire;

/// Insets (a safe area); non-finite or negative ones become zero.
pub fn insets(a: kivact_types::Insets) -> wire::Insets {
    let side = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
    wire::Insets {
        top: side(a.top),
        right: side(a.right),
        bottom: side(a.bottom),
        left: side(a.left),
    }
}

/// Arvane's line height for a protocol one: unset and invalid values
/// (not finite, not above zero) are `Normal`.
pub(crate) fn line_height(p: Option<wire::LineHeight>) -> kivact_types::LineHeight {
    let positive = |v: f32| v.is_finite() && v > 0.0;
    match p {
        Some(wire::LineHeight::Scale(v)) if positive(v) => kivact_types::LineHeight::Scale(v),
        Some(wire::LineHeight::Px(v)) if positive(v) => kivact_types::LineHeight::Px(v),
        _ => kivact_types::LineHeight::Normal,
    }
}
pub(crate) fn text(p: &wire::ComputedText) -> kivact_types::protocol::TextStyle {
    let positive = |v: f32| v.is_finite() && v > 0.0;
    let font_size = if positive(p.font_size) {
        p.font_size
    } else {
        kivact_types::protocol::TextStyle::default().font_size
    };
    let mut style = kivact_types::protocol::TextStyle {
        font_size,
        weight: p.weight,
        color: color(p.color),
        line_height: line_height(p.line_height),
        wrap: p.wrap,
        align: p.align,
        family: p.family.clone(),
        italic: p.italic,
        ..kivact_types::protocol::TextStyle::default()
    };
    if let Some(m) = p.more.as_deref() {
        style.letter_spacing = finite(m.letter_spacing.px(font_size));
        style.word_spacing = finite(m.word_spacing.px(font_size));
        style.kerning = m.kerning;
        style.features = m.features.clone();
        style.decoration = decoration(m.decoration, font_size);
        style.overflow = m.overflow;
        style.max_lines = m.max_lines;
        style.capsize = m.capsize;
        style.transform = m.transform;
    }
    style
}

/// A finite length (non-finite ones become zero).
fn finite(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

/// A decoration of text of `font_size` (its em lengths resolved).
pub(crate) fn decoration(
    d: wire::TextDecoration,
    font_size: f32,
) -> kivact_types::protocol::TextDecoration {
    kivact_types::protocol::TextDecoration {
        lines: d.lines,
        color: d.color.map(color),
        thickness: d
            .thickness
            .map(|t| t.px(font_size))
            .filter(|t| t.is_finite() && *t > 0.0),
        underline_offset: finite(d.underline_offset.px(font_size)),
    }
}
/// The spans of a rich text of `base` style (`font_size`: its resolved
/// size) as Arvane spans: em lengths resolved against each span's size,
/// colors drawn through `tint`, families resolved by `family`; a span
/// with a background is highlighted.
pub fn spans(
    spans: &[wire::TextSpan],
    font_size: f32,
    tint: impl Fn(wire::Rgba) -> kivact_types::Rgba,
    mut family: impl FnMut(&str) -> Option<String>,
) -> Vec<kivact_types::protocol::TextSpan> {
    spans
        .iter()
        .map(|span| {
            let s = &span.style;
            let size = s
                .font_size
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(font_size);
            kivact_types::protocol::TextSpan {
                range: span.range.clone(),
                style: kivact_types::protocol::SpanStyle {
                    font_size: s.font_size.map(|_| size),
                    weight: s.weight,
                    color: s.color.map(&tint),
                    family: s.family.as_deref().and_then(&mut family),
                    italic: s.italic,
                    letter_spacing: s.letter_spacing.map(|l| finite(l.px(size))),
                    word_spacing: s.word_spacing.map(|l| finite(l.px(size))),
                    kerning: s.kerning,
                    features: s.features.clone(),
                    decoration: s.decoration.map(|d| {
                        let color = d.color.map(&tint);
                        kivact_types::protocol::TextDecoration {
                            color,
                            ..decoration(d, size)
                        }
                    }),
                    highlight: s.background.is_some(),
                },
            }
        })
        .collect()
}
