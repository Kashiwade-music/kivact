//! Role: Axis mapping between data values and pixels, and the automatic view range.

use super::model::{AxisScale, PlotModel, PlotSeriesKind, PlotView};

/// A visible range `lo..hi` of one axis, mapped onto `start..start + span`
/// pixels. A negative span runs the axis upward (the y axis).
#[derive(Clone, Copy, Debug)]
pub(super) struct Map {
    scale: AxisScale,
    lo: f64,
    hi: f64,
    start: f64,
    span: f64,
}

impl Map {
    /// The map of `lo..hi` onto pixels from `start` by `span`.
    pub(super) fn new(scale: AxisScale, lo: f64, hi: f64, start: f64, span: f64) -> Self {
        Self {
            scale,
            lo,
            hi,
            start,
            span,
        }
    }

    /// The axis transform: the identity, or log10.
    fn transform(scale: AxisScale, value: f64) -> f64 {
        match scale {
            AxisScale::Linear => value,
            AxisScale::Log10 => value.log10(),
        }
    }

    /// The inverse of [`transform`](Self::transform).
    fn inverse(scale: AxisScale, value: f64) -> f64 {
        match scale {
            AxisScale::Linear => value,
            AxisScale::Log10 => 10f64.powf(value),
        }
    }

    /// The transformed ends of the visible range.
    fn ends(&self) -> (f64, f64) {
        (
            Self::transform(self.scale, self.lo),
            Self::transform(self.scale, self.hi),
        )
    }

    /// The pixel of `value`. NaN and values a log axis cannot show give
    /// `None`. Infinities sit one pixel beyond the visible edge on their side
    /// (`-inf` before the start, `+inf` beyond the end); a log axis draws no
    /// `-inf`.
    pub(super) fn px(&self, value: f64) -> Option<f64> {
        if value.is_nan() {
            return None;
        }
        let direction = self.span.signum();
        if value == f64::INFINITY {
            return Some(self.start + self.span + direction);
        }
        if value == f64::NEG_INFINITY {
            return match self.scale {
                AxisScale::Linear => Some(self.start - direction),
                AxisScale::Log10 => None,
            };
        }
        if self.scale == AxisScale::Log10 && value <= 0.0 {
            return None;
        }
        Some(self.start + self.fraction(value) * self.span)
    }

    /// The value at pixel `px`.
    pub(super) fn value(&self, px: f64) -> f64 {
        self.at_fraction((px - self.start) / self.span)
    }

    /// The value at the fraction `t` of the range (0 at `lo`, 1 at `hi`).
    pub(super) fn at_fraction(&self, t: f64) -> f64 {
        let (a, b) = self.ends();
        Self::inverse(self.scale, a + t * (b - a))
    }

    /// The fraction of the range that `value` lies at (outside 0..1 beyond
    /// the ends). A range of zero width gives one half.
    pub(super) fn fraction(&self, value: f64) -> f64 {
        let (a, b) = self.ends();
        if b == a {
            0.5
        } else {
            (Self::transform(self.scale, value) - a) / (b - a)
        }
    }
}

/// The automatic view of `model`: the finite values of every series (on a
/// log axis, the positive ones), padded by 5% on each side in the axis's own
/// scale. With no values the range is `0..1` (`1..10` on a log axis). A range
/// of zero width widens by half a unit in the axis's scale (a factor of √10
/// on a log axis).
pub fn auto_view(model: &PlotModel) -> PlotView {
    let (x_min, x_max) = padded(extent(model, Axis::X, model.x.scale), model.x.scale);
    let (y_min, y_max) = padded(extent(model, Axis::Y, model.y.scale), model.y.scale);
    PlotView {
        x_min,
        x_max,
        y_min,
        y_max,
    }
}

/// Which axis of the plot.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Axis {
    X,
    Y,
}

/// The smallest and largest usable value of one axis over all series.
pub(super) fn extent(model: &PlotModel, axis: Axis, scale: AxisScale) -> Option<(f64, f64)> {
    let mut range: Option<(f64, f64)> = None;
    let mut take = |values: &[f64]| {
        for &v in values {
            if !v.is_finite() || (scale == AxisScale::Log10 && v <= 0.0) {
                continue;
            }
            range = Some(match range {
                None => (v, v),
                Some((lo, hi)) => (lo.min(v), hi.max(v)),
            });
        }
    };
    for series in &model.series {
        match (&series.kind, axis) {
            (PlotSeriesKind::Line { x, .. }, Axis::X)
            | (PlotSeriesKind::Points { x, .. }, Axis::X)
            | (PlotSeriesKind::Stems { x, .. }, Axis::X) => take(x.as_slice()),
            (PlotSeriesKind::Line { y, .. }, Axis::Y)
            | (PlotSeriesKind::Points { y, .. }, Axis::Y)
            | (PlotSeriesKind::Stems { y, .. }, Axis::Y) => take(y.as_slice()),
            (PlotSeriesKind::Steps { edges, .. }, Axis::X) => take(edges.as_slice()),
            (PlotSeriesKind::Steps { values, .. }, Axis::Y) => take(values.as_slice()),
            (PlotSeriesKind::Heatmap { x_edges, .. }, Axis::X) => take(x_edges.as_slice()),
            (PlotSeriesKind::Heatmap { y_edges, .. }, Axis::Y) => take(y_edges.as_slice()),
        }
    }
    range
}

/// The padded `(min, max)` of `range` on `scale`, or the default range.
fn padded(range: Option<(f64, f64)>, scale: AxisScale) -> (f64, f64) {
    let Some((lo, hi)) = range else {
        return match scale {
            AxisScale::Linear => (0.0, 1.0),
            AxisScale::Log10 => (1.0, 10.0),
        };
    };
    let (mut a, mut b) = match scale {
        AxisScale::Linear => (lo, hi),
        AxisScale::Log10 => (lo.log10(), hi.log10()),
    };
    if a == b {
        a -= 0.5;
        b += 0.5;
    } else {
        let pad = (b - a) * 0.05;
        a -= pad;
        b += pad;
    }
    match scale {
        AxisScale::Linear => (a, b),
        AxisScale::Log10 => (10f64.powf(a), 10f64.powf(b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plot::model::{
        LegendPosition, LineDash, PlotAxis, PlotColor, PlotData, PlotSeries, TickFormat,
    };

    fn model(x: &[f64], y: &[f64], scale: AxisScale) -> PlotModel {
        PlotModel {
            title: String::new(),
            x: PlotAxis {
                label: String::new(),
                scale,
                format: TickFormat::Plain,
            },
            y: PlotAxis {
                label: String::new(),
                scale: AxisScale::Linear,
                format: TickFormat::Plain,
            },
            series: vec![PlotSeries {
                name: "s".into(),
                color: PlotColor::Palette(0),
                width: 1.0,
                dash: LineDash::Solid,
                kind: PlotSeriesKind::Line {
                    x: PlotData::new(x.to_vec()),
                    y: PlotData::new(y.to_vec()),
                },
            }],
            legend: LegendPosition::TopRight,
        }
    }

    #[test]
    fn an_empty_model_has_the_default_ranges() {
        let mut empty = model(&[], &[], AxisScale::Linear);
        empty.series.clear();
        let view = auto_view(&empty);
        assert_eq!(
            (view.x_min, view.x_max, view.y_min, view.y_max),
            (0.0, 1.0, 0.0, 1.0)
        );
        empty.x.scale = AxisScale::Log10;
        let view = auto_view(&empty);
        assert!((view.x_min - 1.0).abs() < 1e-12 && (view.x_max - 10.0).abs() < 1e-12);
    }

    #[test]
    fn a_linear_range_is_padded_by_five_percent() {
        let view = auto_view(&model(&[0.0, 10.0], &[2.0, 4.0], AxisScale::Linear));
        assert!((view.x_min + 0.5).abs() < 1e-12 && (view.x_max - 10.5).abs() < 1e-12);
        assert!((view.y_min - 1.9).abs() < 1e-12 && (view.y_max - 4.1).abs() < 1e-12);
    }

    #[test]
    fn a_zero_width_range_widens_by_half_a_unit() {
        let view = auto_view(&model(&[3.0, 3.0], &[5.0, 5.0], AxisScale::Linear));
        assert!((view.x_min - 2.5).abs() < 1e-12 && (view.x_max - 3.5).abs() < 1e-12);
        let log = auto_view(&model(&[100.0, 100.0], &[1.0, 2.0], AxisScale::Log10));
        assert!((log.x_min - 100.0 / 10f64.sqrt()).abs() < 1e-9);
        assert!((log.x_max - 100.0 * 10f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn a_log_axis_ignores_values_that_cannot_be_shown() {
        let x = [-1.0, 0.0, 10.0, 1000.0, f64::NAN];
        let view = auto_view(&model(&x, &[1.0, 2.0, 3.0, 4.0, 5.0], AxisScale::Log10));
        // The data span is 1..3 in log10 space; the padding is 5% of it.
        assert!((view.x_min - 10f64.powf(1.0 - 0.1)).abs() < 1e-9);
        assert!((view.x_max - 10f64.powf(3.0 + 0.1)).abs() < 1e-9);
    }

    #[test]
    fn values_map_to_pixels_and_back() {
        let map = Map::new(AxisScale::Log10, 1.0, 100.0, 10.0, 200.0);
        assert!((map.px(10.0).unwrap_or(f64::NAN) - 110.0).abs() < 1e-9);
        assert!((map.value(110.0) - 10.0).abs() < 1e-9);
        assert_eq!(map.px(0.0), None);
        assert_eq!(map.px(f64::NAN), None);
        let up = Map::new(AxisScale::Linear, 0.0, 1.0, 100.0, -100.0);
        assert!(up.px(1.0).unwrap_or(f64::NAN).abs() < 1e-9);
        assert_eq!(up.px(f64::NEG_INFINITY), Some(101.0));
        assert_eq!(up.px(f64::INFINITY), Some(-1.0));
    }
}
