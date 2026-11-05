//! Role: Draw segmented gauges and rings from the standard progress owner's values.
use crate::{Tokens, display::ToneExt};
use kivact::draw::{Path, Stroke};
use kivact::geometry::Point;
use kivact::state::Value;
use kivact::style::{Align, Color, ColorToken, Direction, Justify, Style, Theme, style};
use kivact::ui::{Look, ProgressState, ProgressVariant};

/// Application-owned segment count for a standard gauge, reactive when needed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaugeSegments(pub Value<usize>);
impl kivact::ui::Variant for GaugeSegments {}

pub(crate) fn special(t: &Theme, s: &Look<ProgressState>) -> Option<Style> {
    match s.variant::<ProgressVariant>() {
        Some(ProgressVariant::Gauge) => Some(gauge(t, s)),
        Some(ProgressVariant::Ring) => Some(ring(t, s)),
        _ => None,
    }
}
pub(crate) fn special_fill(s: &Look<ProgressState>) -> Option<Style> {
    matches!(
        s.variant::<ProgressVariant>(),
        Some(ProgressVariant::Gauge | ProgressVariant::Ring)
    )
    .then(|| style().size(0, 0).min_width(0))
}
pub(crate) fn label(t: &Theme, _: &Look<ProgressState>) -> Style {
    style()
        .font_family(t.ext::<Tokens>().display_family.as_str())
        .font_size(24)
        .weight(700)
        .italic(true)
}
fn gauge(t: &Theme, s: &Look<ProgressState>) -> Style {
    let count = s
        .variant::<GaugeSegments>()
        .map_or(100, |count| count.0.get());
    let filled = (s.value.unwrap_or(0.) * count as f64).round();
    let color = s
        .variant::<kivact::interaction::Tone>()
        .copied()
        .unwrap_or_default()
        .color(t);
    let empty = t.ext::<Tokens>().surface_3;
    let skew = 9.0 * 12.0_f64.to_radians().tan();
    style()
        .width(count.saturating_mul(15).saturating_sub(3) as f64)
        .height(if count == 0 { 0 } else { 18 })
        .clip(false)
        .background(Color::TRANSPARENT)
        .paint(move |d, _| {
            for i in 0..count {
                let x = (i * 15) as f64;
                d.fill_path(
                    &Path::new()
                        .move_to(Point::new(x + skew, 0.))
                        .line_to(Point::new(x + 12.0 + skew, 0.))
                        .line_to(Point::new(x + 12.0 - skew, 18.))
                        .line_to(Point::new(x - skew, 18.))
                        .close(),
                    if (i as f64) < filled { color } else { empty },
                );
            }
        })
}
fn ring(t: &Theme, s: &Look<ProgressState>) -> Style {
    let shown = s.shown;
    let tone = s
        .variant::<kivact::interaction::Tone>()
        .copied()
        .unwrap_or_default();
    let color = tone.color(t);
    let empty = t.ext::<Tokens>().surface_3;
    style()
        .size(88, 88)
        .clip(false)
        .background(Color::TRANSPARENT)
        .direction(Direction::ColumnReverse)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(0)
        .paint(move |d, _| {
            let center = Point::new(44., 44.);
            d.stroke_path(
                &Path::new().arc(center, 36., 0., 359.9, false),
                &Stroke::new(8),
                empty,
            );
            let end = -90. + shown.get() * 360.;
            if end > -90. {
                d.stroke_path(
                    &Path::new().arc(center, 36., -90., end, false),
                    &Stroke::new(8),
                    color,
                );
            }
            d.stroke_path(
                &Path::new().arc(center, 36., 0., 359.9, false),
                &Stroke::new(9).dash([1.2, 8.62]),
                ColorToken::Surface,
            );
        })
}
