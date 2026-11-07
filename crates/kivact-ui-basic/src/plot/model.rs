//! Role: The plot's data model: shared number arrays, series, axes, legend and view ranges.

use std::sync::Arc;

/// A numeric array shared by reference. Cloning is cheap, and two values are
/// equal when they point at the same array (so a reactive value that is
/// rebuilt from the same data does not redraw).
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

impl PartialEq for PlotData {
    /// Equal when both point at the same array.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.values, &other.values)
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
#[derive(Clone, PartialEq, Debug)]
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

/// A series color: an entry of the theme's chart palette, or a fixed RGB value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlotColor {
    /// The index into the theme's chart colors, taken modulo their count.
    Palette(u32),
    /// A color as `0xRRGGBB`.
    Rgb(u32),
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
#[derive(Clone, PartialEq)]
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
    /// map `z_min..z_max` onto the colormap; NaN cells are transparent.
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
#[derive(Clone, PartialEq)]
pub struct PlotSeries {
    /// The name shown in the legend and in readouts.
    pub name: String,
    /// The color.
    pub color: PlotColor,
    /// The line width in logical px (lines and stems).
    pub width: f64,
    /// The pattern of lines and stems.
    pub dash: LineDash,
    /// What it draws.
    pub kind: PlotSeriesKind,
}

/// Where the legend is drawn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegendPosition {
    /// In the top right corner of the plot.
    TopRight,
    /// Not drawn.
    Hidden,
}

/// Everything a plot draws: title, axes, series and legend.
#[derive(Clone, PartialEq)]
pub struct PlotModel {
    /// The title; empty for none.
    pub title: String,
    /// The horizontal axis.
    pub x: PlotAxis,
    /// The vertical axis.
    pub y: PlotAxis,
    /// The series, drawn in order.
    pub series: Vec<PlotSeries>,
    /// The legend.
    pub legend: LegendPosition,
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
    /// Fit the data ([`auto_view`](super::auto_view)).
    Auto,
    /// The given range.
    Manual(PlotView),
}
