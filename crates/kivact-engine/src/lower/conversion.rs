//! Role: Semantic conversion of shared values to neutral text and geometry.
//!
//! Shared kivact-types values pass through unchanged. Only contracts whose
//! representations differ are resolved here: safe areas, text lengths,
//! shaping styles and spans. Tree layout and input admission stay with callers.
//!
//! Invariants:
//! - The checks of these values stay on the host side of the boundary: a
//!   line height must be positive and finite (else `Normal`; Arvane gets
//!   one of its three values), colors use `Rgba::sanitize`,
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
use kivact_types as types;

/// Arvane's line height for a protocol one: unset and invalid values
/// (not finite, not above zero) are `Normal`.
pub(crate) fn line_height(p: Option<types::LineHeight>) -> kivact_types::LineHeight {
    let positive = |v: f64| v.is_finite() && v > 0.0;
    match p {
        Some(types::LineHeight::Scale(v)) if positive(v) => kivact_types::LineHeight::Scale(v),
        Some(types::LineHeight::Px(v)) if positive(v) => kivact_types::LineHeight::Px(v),
        _ => kivact_types::LineHeight::Normal,
    }
}
/// A finite length (non-finite ones become zero).
fn finite(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}

/// The spans of a rich text of `base` style (`font_size`: its resolved
/// size) as Arvane spans: em lengths resolved against each span's size,
/// colors drawn through `tint`, families resolved by `family`; a span
/// with a background is highlighted.
pub(crate) fn spans(
    spans: &[types::TextSpan],
    font_size: f64,
    tint: impl Fn(types::Rgba) -> kivact_types::Rgba,
    mut family: impl FnMut(&str) -> Option<String>,
) -> Vec<arvane_protocol::ResolvedSpan> {
    spans
        .iter()
        .map(|span| {
            let s = &span.style;
            let size = s
                .font_size
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(font_size);
            arvane_protocol::ResolvedSpan {
                range: span.range.clone(),
                style: arvane_protocol::ResolvedSpanStyle {
                    font_size: s.font_size.map(|_| size),
                    weight: s.weight,
                    color: s.color.map(&tint),
                    family: s.family.as_deref().and_then(&mut family),
                    italic: s.italic,
                    letter_spacing: s.letter_spacing.map(|l| finite(l.px(size))),
                    word_spacing: s.word_spacing.map(|l| finite(l.px(size))),
                    kerning: s.kerning,
                    features: span_features(s),
                    variations: s.variations.clone(),
                    font_width: s.font_width,
                    stroke: s
                        .stroke
                        .map(|stroke| (finite(stroke.width.px(size)).max(0.0), tint(stroke.color))),
                    decoration: s.span_decoration().map(|d| {
                        let color = d.color.map(&tint);
                        kivact_types::TextDecoration { color, ..d }
                    }),
                    highlight: span.background.is_some(),
                },
            }
        })
        .collect()
}

/// A span's OpenType features: its own, then `tnum` when it sets tabular
/// numbers (as a text's features take it); `None` when it sets neither.
fn span_features(s: &types::TextStyle) -> Option<Vec<types::FontFeature>> {
    (s.features.is_some() || s.tabular_nums.is_some()).then(|| {
        let mut out = s.features.clone().unwrap_or_default();
        if s.tabular_nums == Some(true) {
            out.push(types::FontFeature {
                tag: *b"tnum",
                value: 1,
            });
        }
        out
    })
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the span conversion.
    use super::*;

    #[test]
    fn a_span_keeps_its_decoration_style() {
        let span = types::TextSpan {
            range: 0..1,
            style: types::TextStyle {
                decoration_lines: Some(types::DecorationLines::UNDERLINE),
                decoration_style: Some(types::TextDecorationStyle::Wavy),
                ..types::TextStyle::default()
            },
            background: None,
        };
        let out = spans(&[span], 16.0, |color| color, |_| None);
        let decoration = out[0].style.decoration.expect("the span is decorated");
        assert_eq!(decoration.style, types::TextDecorationStyle::Wavy);
    }
}
