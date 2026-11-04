//! Role: Paint and hit-test the pie, donut and radial charts.

use super::model::{ChartKind, Frame, Hit, HitRow, series_color};
use super::polar::{disc, locate, polar};
use crate::tokens::Tokens;
use kivact::draw::{Draw, Path, Stroke, TextBaseline, TextFormat};
use kivact::geometry::{Point, Size};
use kivact::style::TextAlign;

/// The inner radius of a donut and of the innermost radial ring, against the outer radius.
const DONUT_INNER: f64 = 0.6;
const RADIAL_INNER: f64 = 0.3;
/// The gap between radial rings.
const RING_GAP: f64 = 4.0;
/// The angle the largest radial ring sweeps.
const RADIAL_SWEEP: f64 = 270.0;
/// How far the hovered slice grows.
const POP: f64 = 4.0;
/// The angle where slices and rings start: the top.
const START: f64 = -90.0;

/// The slice sizes: the sum of each series, never negative.
fn totals(f: &Frame) -> Vec<f64> {
    (0..f.config.series.len())
        .map(|series| f.total(series).max(0.0))
        .collect()
}

/// Paint the chart; `hit` is the slice or ring under the pointer.
pub(super) fn paint(d: &mut Draw, size: Size, f: &Frame, hit: Option<&Hit>) {
    let tokens = d.theme().ext::<Tokens>();
    let hovered = hit.map(|hit| hit.index);
    if f.kind == ChartKind::Radial {
        paint_rings(d, size, f, &tokens, hovered);
    } else {
        paint_slices(d, size, f, &tokens, hovered);
    }
}

fn paint_slices(d: &mut Draw, size: Size, f: &Frame, tokens: &Tokens, hovered: Option<usize>) {
    let (center, radius) = disc(size);
    let donut = f.kind == ChartKind::Donut;
    let inner = if donut { radius * DONUT_INNER } else { 0.0 };
    let totals = totals(f);
    let sum: f64 = totals.iter().sum();
    if sum <= 0.0 {
        return;
    }
    let mut from = START;
    for (series, total) in totals.iter().enumerate() {
        let to = from + (360.0 * total / sum);
        let outer = if hovered == Some(series) {
            radius + POP
        } else {
            radius
        };
        if *total > 0.0 {
            let mut slice = if donut {
                Path::new().arc(center, outer, from, to, false)
            } else {
                Path::new()
                    .move_to(center)
                    .line_to(polar(center, outer, from))
                    .arc(center, outer, from, to, false)
            };
            if donut {
                slice = slice
                    .line_to(polar(center, inner, to))
                    .arc(center, inner, to, from, true);
            }
            d.fill_path(&slice.close(), series_color(f.config, series, tokens));
        }
        from = to;
    }
    if donut {
        let value = TextFormat {
            size: 24.0,
            weight: 700,
            color: tokens.foreground,
            align: TextAlign::Center,
            baseline: TextBaseline::Middle,
            ..TextFormat::default()
        };
        d.text(&f.format(sum), Point::new(center.x, center.y - 8.0), value);
        let label = TextFormat {
            size: 12.0,
            color: tokens.muted_foreground,
            align: TextAlign::Center,
            baseline: TextBaseline::Middle,
            ..TextFormat::default()
        };
        d.text(f.total_label, Point::new(center.x, center.y + 14.0), label);
    }
}

/// The ring geometry: the middle radius and thickness of ring `series` of `count`.
fn ring(radius: f64, count: usize, series: usize) -> (f64, f64) {
    let inner = radius * RADIAL_INNER;
    let band = (radius - inner) / count.max(1) as f64;
    (
        radius - band * (series as f64 + 0.5),
        (band - RING_GAP).max(2.0),
    )
}

fn paint_rings(d: &mut Draw, size: Size, f: &Frame, tokens: &Tokens, hovered: Option<usize>) {
    let (center, radius) = disc(size);
    let totals = totals(f);
    let largest = totals.iter().copied().fold(0.0, f64::max);
    for (series, total) in totals.iter().enumerate() {
        let (middle, thickness) = ring(radius, totals.len(), series);
        let thickness = if hovered == Some(series) {
            thickness + POP
        } else {
            thickness
        };
        let track = Path::new().arc(center, middle, START, START + RADIAL_SWEEP, false);
        d.stroke_path(&track, &Stroke::new(thickness), tokens.muted);
        if largest > 0.0 && *total > 0.0 {
            let sweep = RADIAL_SWEEP * (total / largest);
            let value = Path::new().arc(center, middle, START, START + sweep, false);
            d.stroke_path(
                &value,
                &Stroke::new(thickness),
                series_color(f.config, series, tokens),
            );
        }
    }
}

/// The slice or ring under `at`, with its series' total.
pub(super) fn hit(size: Size, f: &Frame, at: Point) -> Option<Hit> {
    let (center, radius) = disc(size);
    let (distance, angle) = locate(center, at);
    let totals = totals(f);
    let series = match f.kind {
        ChartKind::Radial => {
            let inner = radius * RADIAL_INNER;
            if distance > radius || distance < inner {
                return None;
            }
            let band = (radius - inner) / totals.len().max(1) as f64;
            Some((((radius - distance) / band) as usize).min(totals.len().saturating_sub(1)))
        }
        kind => {
            let inner = if kind == ChartKind::Donut {
                radius * DONUT_INNER
            } else {
                0.0
            };
            let sum: f64 = totals.iter().sum();
            if distance > radius + POP || distance < inner || sum <= 0.0 {
                return None;
            }
            let mut end = 0.0;
            totals.iter().position(|total| {
                end += 360.0 * total / sum;
                (angle) < end && *total > 0.0
            })
        }
    }?;
    let config = f.config.series.get(series)?;
    Some(Hit {
        index: series,
        title: None,
        rows: vec![HitRow {
            series,
            label: config.label.clone(),
            value: totals[series],
        }],
    })
}
