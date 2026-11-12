//! Role: Series: what a plot draws, its data, color, line style and heatmap range.

use super::data::PlotData;
use kivact::style::Color;

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

/// The shape of a series and its data. The constructors of [`PlotSeries`]
/// build it, so the shape cannot be written in an inconsistent way.
#[derive(Clone, PartialEq, Debug)]
pub(super) enum PlotSeriesKind {
    /// A polyline through `(x, y)`. Hover reads the nearest point only when `x` is ascending.
    Line { x: PlotData, y: PlotData },
    /// Circles at `(x, y)`.
    Points { x: PlotData, y: PlotData },
    /// A vertical line from the bottom of the plot to each `y`, with a dot at its top.
    Stems { x: PlotData, y: PlotData },
    /// A histogram: the bar of `values[i]` spans `edges[i]..edges[i + 1]`.
    Steps { edges: PlotData, values: PlotData },
    /// A colored grid. `z` is row-major with `(y_edges.len() - 1)` rows of
    /// `(x_edges.len() - 1)` values; NaN cells are transparent. `z_range` maps
    /// its values onto the colors; `None` takes the finite range of `z`.
    Heatmap {
        x_edges: PlotData,
        y_edges: PlotData,
        z: PlotData,
        z_range: Option<(f64, f64)>,
    },
}

/// One series of a plot. Create it with one constructor for its kind, then
/// adjust it with the setters. A setter that does not apply to the kind is
/// ignored, and each setter's documentation says which kinds it applies to.
#[derive(Clone, PartialEq, Debug)]
pub struct PlotSeries {
    /// The name shown in the legend and in the description.
    pub(super) name: String,
    /// The color; `None` takes the kit's chart color of the series' index.
    pub(super) color: Option<Color>,
    /// The line width in logical px.
    pub(super) width: f64,
    /// The pattern of lines and stems.
    pub(super) dash: LineDash,
    /// The radius of points and of the dots at the top of stems, in logical px.
    pub(super) radius: f64,
    /// What it draws.
    pub(super) kind: PlotSeriesKind,
}

impl PlotSeries {
    /// A line named `name` through `(x[i], y[i])`, 2 px wide and solid.
    pub fn line(name: impl Into<String>, x: impl Into<PlotData>, y: impl Into<PlotData>) -> Self {
        Self::of(
            name,
            PlotSeriesKind::Line {
                x: x.into(),
                y: y.into(),
            },
        )
    }

    /// Circles named `name` at `(x[i], y[i])`, of radius 3 px.
    pub fn points(name: impl Into<String>, x: impl Into<PlotData>, y: impl Into<PlotData>) -> Self {
        Self::of(
            name,
            PlotSeriesKind::Points {
                x: x.into(),
                y: y.into(),
            },
        )
    }

    /// Stems named `name`: a line from the bottom of the plot to each
    /// `(x[i], y[i])`, with a dot of radius 3 px at its top.
    pub fn stems(name: impl Into<String>, x: impl Into<PlotData>, y: impl Into<PlotData>) -> Self {
        Self::of(
            name,
            PlotSeriesKind::Stems {
                x: x.into(),
                y: y.into(),
            },
        )
    }

    /// A histogram named `name`: the bar of `values[i]` spans
    /// `edges[i]..edges[i + 1]`, so `edges` has one value more than `values`.
    pub fn steps(
        name: impl Into<String>,
        edges: impl Into<PlotData>,
        values: impl Into<PlotData>,
    ) -> Self {
        Self::of(
            name,
            PlotSeriesKind::Steps {
                edges: edges.into(),
                values: values.into(),
            },
        )
    }

    /// A heatmap named `name` of cells between `x_edges` and `y_edges`, with
    /// the values `z` row by row. It has `y_edges.len() - 1` rows and
    /// `x_edges.len() - 1` columns, so `z` must hold `rows × cols` values. A
    /// `z` of any other length draws nothing and reads no value.
    pub fn heatmap(
        name: impl Into<String>,
        x_edges: impl Into<PlotData>,
        y_edges: impl Into<PlotData>,
        z: impl Into<PlotData>,
    ) -> Self {
        Self::of(
            name,
            PlotSeriesKind::Heatmap {
                x_edges: x_edges.into(),
                y_edges: y_edges.into(),
                z: z.into(),
                z_range: None,
            },
        )
    }

    /// The defaults of a series of `kind`.
    fn of(name: impl Into<String>, kind: PlotSeriesKind) -> Self {
        Self {
            name: name.into(),
            color: None,
            width: 2.0,
            dash: LineDash::Solid,
            radius: 3.0,
            kind,
        }
    }

    /// The same series in `color`. Applies to every kind.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// The same series with lines of `width` logical px. Applies to lines and stems.
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// The same series with lines in the pattern `dash`. Applies to lines and stems.
    pub fn dash(mut self, dash: LineDash) -> Self {
        self.dash = dash;
        self
    }

    /// The same series with circles of `radius` logical px: the points, and
    /// the dots at the top of stems. Applies to points and stems.
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// The same heatmap with `min` mapped to the first color and `max` to the
    /// last. Without it the range is the smallest and largest finite value of
    /// `z`. Applies to heatmaps; on any other kind it changes nothing.
    pub fn z_range(mut self, min: f64, max: f64) -> Self {
        if let PlotSeriesKind::Heatmap { z_range, .. } = &mut self.kind {
            *z_range = Some((min, max));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the series defaults and the setters.

    use super::*;

    #[test]
    fn a_series_has_one_constructor_per_kind_and_defaults_to_its_chart_color() {
        let line = PlotSeries::line("Gain", vec![0.0], vec![1.0]);
        assert_eq!(line.color, None);
        assert_eq!((line.width, line.radius), (2.0, 3.0));
        assert_eq!(line.dash, LineDash::Solid);
        assert!(matches!(line.kind, PlotSeriesKind::Line { .. }));
        let points = PlotSeries::points("p", [0.0], [1.0]);
        assert!(matches!(points.kind, PlotSeriesKind::Points { .. }));
        let stems = PlotSeries::stems("s", [0.0], [1.0]);
        assert!(matches!(stems.kind, PlotSeriesKind::Stems { .. }));
        let steps = PlotSeries::steps("h", [0.0, 1.0], [2.0]);
        assert!(matches!(steps.kind, PlotSeriesKind::Steps { .. }));
        let grid = PlotSeries::heatmap("g", [0.0, 1.0], [0.0, 1.0], [2.0]);
        assert!(matches!(grid.kind, PlotSeriesKind::Heatmap { .. }));
    }

    #[test]
    fn setters_replace_the_defaults() {
        let series = PlotSeries::line("Gain", [0.0], [1.0])
            .width(4.0)
            .dash(LineDash::Dotted)
            .radius(5.0);
        assert_eq!((series.width, series.radius), (4.0, 5.0));
        assert_eq!(series.dash, LineDash::Dotted);
    }

    #[test]
    fn z_range_applies_to_heatmaps_only() {
        let grid = PlotSeries::heatmap("g", [0.0, 1.0], [0.0, 1.0], [2.0]).z_range(-1.0, 1.0);
        assert!(matches!(
            grid.kind,
            PlotSeriesKind::Heatmap {
                z_range: Some((min, max)),
                ..
            } if min == -1.0 && max == 1.0
        ));
        let line = PlotSeries::line("l", [0.0], [1.0]).z_range(-1.0, 1.0);
        assert_eq!(line, PlotSeries::line("l", [0.0], [1.0]));
    }
}
