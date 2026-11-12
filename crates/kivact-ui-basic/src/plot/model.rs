//! Role: The plot model: axes, series and legend, and the range they fit.

use super::axis::{AxisScale, PlotAxis};
use super::series::{PlotSeries, PlotSeriesKind};

/// Everything a plot draws: title, axes, series and legend. Build it with
/// [`PlotModel::new`] and the setters.
#[derive(Clone, PartialEq, Debug)]
pub struct PlotModel {
    /// The title; empty for none.
    pub(super) title: String,
    /// The horizontal axis.
    pub(super) x: PlotAxis,
    /// The vertical axis.
    pub(super) y: PlotAxis,
    /// The series, drawn in order.
    pub(super) series: Vec<PlotSeries>,
    /// Whether the legend is drawn in the top right corner.
    pub(super) legend: bool,
}

impl PlotModel {
    /// A model of two axes with no title, no series and a legend.
    pub fn new(x: PlotAxis, y: PlotAxis) -> Self {
        Self {
            title: String::new(),
            x,
            y,
            series: Vec::new(),
            legend: true,
        }
    }

    /// The same model with the title `title`.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// The same model with `series` added after the others.
    pub fn series(mut self, series: PlotSeries) -> Self {
        self.series.push(series);
        self
    }

    /// The same model with the legend shown (`true`) or not.
    pub fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
        self
    }

    /// The range that fits the data of every series, padded by 5% on each side.
    pub fn auto_view(&self) -> PlotView {
        let (x_min, x_max) = padded(extent(self, Axis::X, self.x.scale), self.x.scale);
        let (y_min, y_max) = padded(extent(self, Axis::Y, self.y.scale), self.y.scale);
        PlotView {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// The visible range of this model under `range`: the fit of its data
    /// for [`PlotRange::Auto`], else the range given.
    pub(super) fn view_of(&self, range: PlotRange) -> PlotView {
        match range {
            PlotRange::Auto => self.auto_view(),
            PlotRange::Manual(view) => view,
        }
    }
}

kivact::state::constant_value!(PlotModel, PlotRange);

/// The visible range, in data values. On a log axis these are values too,
/// not exponents.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PlotView {
    /// The smallest visible x.
    pub x_min: f64,
    /// The largest visible x.
    pub x_max: f64,
    /// The smallest visible y.
    pub y_min: f64,
    /// The largest visible y.
    pub y_max: f64,
}

/// Whether the range follows the data or is fixed.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlotRange {
    /// Fit the data ([`PlotModel::auto_view`]).
    Auto,
    /// The given range.
    Manual(PlotView),
}

/// Which axis of the plot.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

/// The smallest and largest usable value of one axis over all series.
fn extent(model: &PlotModel, axis: Axis, scale: AxisScale) -> Option<(f64, f64)> {
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
    //! Role: Tests for the automatic view range and the range of a model.

    use super::*;

    fn model(x: &[f64], y: &[f64], scale: AxisScale) -> PlotModel {
        PlotModel::new(PlotAxis::new("x").scale(scale), PlotAxis::new("y"))
            .series(PlotSeries::line("s", x.to_vec(), y.to_vec()))
    }

    #[test]
    fn an_empty_model_has_the_default_ranges() {
        let empty = PlotModel::new(PlotAxis::new("x"), PlotAxis::new("y"));
        let view = empty.auto_view();
        assert_eq!(
            (view.x_min, view.x_max, view.y_min, view.y_max),
            (0.0, 1.0, 0.0, 1.0)
        );
        let log = PlotModel::new(
            PlotAxis::new("x").scale(AxisScale::Log10),
            PlotAxis::new("y"),
        );
        let view = log.auto_view();
        assert!((view.x_min - 1.0).abs() < 1e-12 && (view.x_max - 10.0).abs() < 1e-12);
    }

    #[test]
    fn a_linear_range_is_padded_by_five_percent() {
        let view = model(&[0.0, 10.0], &[2.0, 4.0], AxisScale::Linear).auto_view();
        assert!((view.x_min + 0.5).abs() < 1e-12 && (view.x_max - 10.5).abs() < 1e-12);
        assert!((view.y_min - 1.9).abs() < 1e-12 && (view.y_max - 4.1).abs() < 1e-12);
    }

    #[test]
    fn a_zero_width_range_widens_by_half_a_unit() {
        let view = model(&[3.0, 3.0], &[5.0, 5.0], AxisScale::Linear).auto_view();
        assert!((view.x_min - 2.5).abs() < 1e-12 && (view.x_max - 3.5).abs() < 1e-12);
        let log = model(&[100.0, 100.0], &[1.0, 2.0], AxisScale::Log10).auto_view();
        assert!((log.x_min - 100.0 / 10f64.sqrt()).abs() < 1e-9);
        assert!((log.x_max - 100.0 * 10f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn a_log_axis_ignores_values_that_cannot_be_shown() {
        let x = [-1.0, 0.0, 10.0, 1000.0, f64::NAN];
        let view = model(&x, &[1.0, 2.0, 3.0, 4.0, 5.0], AxisScale::Log10).auto_view();
        // The data span is 1..3 in log10 space; the padding is 5% of it.
        assert!((view.x_min - 10f64.powf(1.0 - 0.1)).abs() < 1e-9);
        assert!((view.x_max - 10f64.powf(3.0 + 0.1)).abs() < 1e-9);
    }

    #[test]
    fn a_range_is_the_fit_or_its_manual_view() {
        let fitted = model(&[0.0, 10.0], &[0.0, 1.0], AxisScale::Linear);
        assert_eq!(fitted.view_of(PlotRange::Auto), fitted.auto_view());
        let view = PlotView {
            x_min: 1.0,
            x_max: 2.0,
            y_min: 3.0,
            y_max: 4.0,
        };
        assert_eq!(fitted.view_of(PlotRange::Manual(view)), view);
    }
}
