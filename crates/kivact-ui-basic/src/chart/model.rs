//! Role: The public chart data types and the frame the kinds paint and hit-test.

use crate::tokens::Tokens;
use kivact::style::Color;

/// One data series of a chart: a key, the name shown in the legend and tooltip, and a color.
///
/// Without a color, series `i` uses chart color `i % 5` of the theme's
/// [`Tokens`] (`--chart-1` to `--chart-5`), so a live theme switch recolors it.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    /// An identifier for the series, such as `"desktop"`.
    pub key: String,
    /// The text shown in the legend and the tooltip.
    pub label: String,
    /// An explicit color; `None` takes the theme's chart color.
    pub color: Option<Color>,
}

impl ChartSeries {
    /// A series with the theme's chart color.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            color: None,
        }
    }

    /// Draw the series in `color` instead of the theme's chart color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// The series of a chart (chart.tsx `ChartConfig`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartConfig {
    /// The series in drawing order; [`ChartRow::values`] holds one value per series.
    pub series: Vec<ChartSeries>,
}

impl ChartConfig {
    /// A configuration of `series`.
    pub fn new(series: impl IntoIterator<Item = ChartSeries>) -> Self {
        Self {
            series: series.into_iter().collect(),
        }
    }
}

/// One category of a chart: its label and one value for each series.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartRow {
    /// The category: the x axis label and the tooltip title.
    pub label: String,
    /// The value of each series, in the order of [`ChartConfig::series`]; missing values are 0.
    pub values: Vec<f64>,
}

impl ChartRow {
    /// A row labelled `label` with `values`.
    pub fn new(label: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            label: label.into(),
            values: values.into_iter().collect(),
        }
    }
}

/// How a [`Chart`](crate::Chart) draws its data.
///
/// Bar, stacked bar, line and area charts draw one mark per series for each
/// row. Pie, donut and radial charts draw one slice or ring per series, sized
/// by the sum of its values over all rows. A radar chart draws one polygon
/// per series with one axis per row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChartKind {
    /// Bars side by side for each row.
    #[default]
    Bar,
    /// Bars stacked on top of each other for each row.
    StackedBar,
    /// A line through the rows for each series.
    Line,
    /// A line with a translucent fill down to zero for each series.
    Area,
    /// Slices of a disc.
    Pie,
    /// Slices of a ring with the total in the middle.
    Donut,
    /// Polygons over a web with one axis per row.
    Radar,
    /// Concentric arcs, the largest series sweeping 270 degrees.
    Radial,
}

/// The data and options of one paint or hit test.
pub(super) struct Frame<'a> {
    pub(super) config: &'a ChartConfig,
    pub(super) rows: &'a [ChartRow],
    pub(super) kind: ChartKind,
    pub(super) horizontal: bool,
    pub(super) grid: bool,
    pub(super) axis: bool,
    /// The caption under the total in the middle of a donut.
    pub(super) total_label: &'a str,
    /// Writes a value as text: axis ticks, the donut total and the tooltip.
    pub(super) format: &'a dyn Fn(f64) -> String,
}

impl Frame<'_> {
    /// `value` as text, through the chart's value format.
    pub(super) fn format(&self, value: f64) -> String {
        (self.format)(value)
    }

    /// The value of series `series` in `row`, 0 when missing or not finite.
    pub(super) fn value(&self, row: usize, series: usize) -> f64 {
        self.rows
            .get(row)
            .and_then(|row| row.values.get(series))
            .copied()
            .filter(|value| value.is_finite())
            .unwrap_or(0.0)
    }

    /// The sum of series `series` over all rows (a slice, ring or legend total).
    pub(super) fn total(&self, series: usize) -> f64 {
        (0..self.rows.len())
            .map(|row| self.value(row, series))
            .sum()
    }
}

/// The color of series `index`: its own or chart color `index % 5`.
pub(super) fn series_color(config: &ChartConfig, index: usize, tokens: &Tokens) -> Color {
    config
        .series
        .get(index)
        .and_then(|series| series.color)
        .unwrap_or(tokens.chart[index % tokens.chart.len()])
}

/// One tooltip row: a series and its value.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct HitRow {
    pub(super) series: usize,
    pub(super) label: String,
    pub(super) value: f64,
}

/// What the pointer is over: the tooltip's content. Where the pointer is
/// is kept apart, so a move within the same row changes neither the
/// picture nor the card's content.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Hit {
    /// The row, slice or ring under the pointer; marks drawn for it are emphasized.
    pub(super) index: usize,
    /// The tooltip title (the row label); none for slices and rings.
    pub(super) title: Option<String>,
    pub(super) rows: Vec<HitRow>,
}
