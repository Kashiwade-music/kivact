//! Role: The plot's data: number arrays, series, axes, legend and range.

use kivact::style::Color;
use std::fmt;
use std::sync::Arc;

/// A numeric array shared by reference. Cloning is cheap. Two values are
/// equal when they share the array or hold the same numbers bit for bit, so a
/// reactive value rebuilt from the same data does not redraw.
#[derive(Clone)]
pub struct PlotData {
    values: Arc<[f64]>,
}

impl PlotData {
    /// The array `values`.
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values: values.into(),
        }
    }

    /// The values in order.
    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }

    /// The number of values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether there are no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl From<Vec<f64>> for PlotData {
    fn from(values: Vec<f64>) -> Self {
        Self::new(values)
    }
}

impl FromIterator<f64> for PlotData {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl PartialEq for PlotData {
    /// Equal when both share one array, or hold the same numbers bit for bit.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.values, &other.values)
            || (self.values.len() == other.values.len()
                && self
                    .values
                    .iter()
                    .zip(other.values.iter())
                    .all(|(a, b)| a.to_bits() == b.to_bits()))
    }
}

impl fmt::Debug for PlotData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlotData(len = {})", self.len())
    }
}

/// How an axis maps values to positions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AxisScale {
    /// Equal steps for equal differences.
    Linear,
    /// Equal steps for equal ratios; values of zero or less are not drawn.
    Log10,
}

/// How tick labels and readouts write their values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TickFormat {
    /// Four significant digits, the shortest form.
    Plain,
    /// SI prefixes: `1.5k`, `2M`, `10m`, `3µ`.
    Si,
    /// Decibels: whole numbers, or one decimal below 1.
    Decibel,
    /// Degrees, with a `°` suffix.
    Degrees,
}

/// One axis: its title, scale and tick format.
#[derive(Clone, PartialEq, Debug)]
pub struct PlotAxis {
    /// The title drawn beside the axis.
    pub label: String,
    /// The scale.
    pub scale: AxisScale,
    /// How ticks and readouts write values.
    pub format: TickFormat,
}

impl PlotAxis {
    /// A linear axis titled `label`, with plain tick labels.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            scale: AxisScale::Linear,
            format: TickFormat::Plain,
        }
    }

    /// The same axis on `scale`.
    pub fn scale(mut self, scale: AxisScale) -> Self {
        self.scale = scale;
        self
    }

    /// The same axis with its tick labels written in `format`.
    pub fn format(mut self, format: TickFormat) -> Self {
        self.format = format;
        self
    }
}

/// The pattern of a line.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineDash {
    /// A continuous line.
    Solid,
    /// Long dashes.
    Dashed,
    /// Short dashes with round caps.
    Dotted,
}

/// What a series draws.
#[derive(Clone, PartialEq, Debug)]
pub enum PlotSeriesKind {
    /// A polyline through `(x, y)`. Hover reads the nearest point only when `x` is ascending.
    Line {
        /// The x values.
        x: PlotData,
        /// The y values, one per x.
        y: PlotData,
    },
    /// Circles of `radius` logical px at `(x, y)`.
    Points {
        /// The x values.
        x: PlotData,
        /// The y values, one per x.
        y: PlotData,
        /// The radius in logical px.
        radius: f64,
    },
    /// A vertical line from the bottom of the plot to each `y`, with a point at its top.
    Stems {
        /// The x values.
        x: PlotData,
        /// The y values, one per x.
        y: PlotData,
    },
    /// A histogram: the bar of `values[i]` spans `edges[i]..edges[i + 1]`.
    Steps {
        /// The bin edges; `edges.len() == values.len() + 1`.
        edges: PlotData,
        /// The bar heights.
        values: PlotData,
    },
    /// A colored grid. `z` is row-major, `rows × cols` values, with
    /// `x_edges.len() == cols + 1` and `y_edges.len() == rows + 1`. The colors
    /// map `z_min..z_max` onto the plot's colormap; NaN cells are transparent.
    Heatmap {
        /// The column edges.
        x_edges: PlotData,
        /// The row edges.
        y_edges: PlotData,
        /// The cell values, row by row.
        z: PlotData,
        /// The number of rows.
        rows: u32,
        /// The number of columns.
        cols: u32,
        /// The value that maps to the first color.
        z_min: f64,
        /// The value that maps to the last color.
        z_max: f64,
    },
}

/// One series of a plot.
#[derive(Clone, PartialEq, Debug)]
pub struct PlotSeries {
    /// The name shown in the legend and in the description.
    pub name: String,
    /// The color; `None` takes the kit's chart color of the series' index.
    pub color: Option<Color>,
    /// The line width in logical px (lines and stems).
    pub width: f64,
    /// The pattern of lines and stems.
    pub dash: LineDash,
    /// What it draws.
    pub kind: PlotSeriesKind,
}

impl PlotSeries {
    /// A solid series of 2 px named `name`, in the chart color of its index.
    pub fn new(name: impl Into<String>, kind: PlotSeriesKind) -> Self {
        Self {
            name: name.into(),
            color: None,
            width: 2.0,
            dash: LineDash::Solid,
            kind,
        }
    }

    /// The same series in `color`.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// The same series with lines of `width` logical px.
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// The same series with lines in the pattern `dash`.
    pub fn dash(mut self, dash: LineDash) -> Self {
        self.dash = dash;
        self
    }
}

/// Everything a plot draws: title, axes, series and legend.
#[derive(Clone, PartialEq, Debug)]
pub struct PlotModel {
    /// The title; empty for none.
    pub title: String,
    /// The horizontal axis.
    pub x: PlotAxis,
    /// The vertical axis.
    pub y: PlotAxis,
    /// The series, drawn in order.
    pub series: Vec<PlotSeries>,
    /// Whether the legend is drawn in the top right corner.
    pub legend: bool,
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

impl PlotRange {
    /// The visible range of `model` under this range: the fit of its data
    /// for `Auto`, else the given range.
    pub fn view(self, model: &PlotModel) -> PlotView {
        match self {
            PlotRange::Auto => model.auto_view(),
            PlotRange::Manual(view) => view,
        }
    }
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
    //! Role: Tests for the data model and the automatic view range.

    use super::*;

    fn model(x: &[f64], y: &[f64], scale: AxisScale) -> PlotModel {
        PlotModel::new(PlotAxis::new("x").scale(scale), PlotAxis::new("y")).series(PlotSeries::new(
            "s",
            PlotSeriesKind::Line {
                x: PlotData::new(x.to_vec()),
                y: PlotData::new(y.to_vec()),
            },
        ))
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
        assert_eq!(PlotRange::Auto.view(&fitted), fitted.auto_view());
        let view = PlotView {
            x_min: 1.0,
            x_max: 2.0,
            y_min: 3.0,
            y_max: 4.0,
        };
        assert_eq!(PlotRange::Manual(view).view(&fitted), view);
    }

    #[test]
    fn data_are_equal_when_bit_for_bit_equal() {
        let a = PlotData::new(vec![0.0, -0.0]);
        assert_eq!(a, a.clone());
        assert_eq!(a, PlotData::new(vec![0.0, -0.0]));
        assert_ne!(a, PlotData::new(vec![0.0, 0.0]));
        assert_eq!(format!("{a:?}"), "PlotData(len = 2)");
    }
}
