//! Role: The Lyra chart: bar, line, area, pie, radar and radial charts on a canvas.

mod cartesian;
mod model;
mod pie;
mod polar;
mod radar;
mod scale;
mod tooltip;

use kivact::control::{Node, NodeBuilder, column, node};
use kivact::draw::{Draw, Point, Size};
use kivact::i18n::{message, message_with};
use kivact::style::{Direction, Length, pct, style};
use kivact::view::{
    IntoValue, IntoView, Orientation, PointerEvent, PointerPhase, Role, View, dynamic, show,
};
use kivact::{Value, memo, signal};
use model::{Frame, Hit};
use scale::format_value;
use std::cell::Cell;
use std::rc::Rc;

pub use model::{ChartConfig, ChartKind, ChartRow, ChartSeries};

/// The options a chart paints and hit-tests with.
#[derive(Clone, Copy)]
struct Options {
    kind: ChartKind,
    horizontal: bool,
    grid: Value<bool>,
    axis: Value<bool>,
}

impl Options {
    fn frame<'a>(
        self,
        config: &'a ChartConfig,
        rows: &'a [ChartRow],
        total_label: &'a str,
        format: &'a dyn Fn(f64) -> String,
    ) -> Frame<'a> {
        Frame {
            config,
            rows,
            kind: self.kind,
            horizontal: self.horizontal,
            grid: self.grid.get(),
            axis: self.axis.get(),
            total_label,
            format,
        }
    }
}

fn paint(d: &mut Draw, size: Size, frame: &Frame, hit: Option<&Hit>) {
    match frame.kind {
        ChartKind::Bar | ChartKind::StackedBar | ChartKind::Line | ChartKind::Area => {
            cartesian::paint(d, size, frame, hit);
        }
        ChartKind::Pie | ChartKind::Donut | ChartKind::Radial => pie::paint(d, size, frame, hit),
        ChartKind::Radar => radar::paint(d, size, frame, hit),
    }
}

fn hit_test(size: Size, frame: &Frame, at: Point) -> Option<Hit> {
    match frame.kind {
        ChartKind::Bar | ChartKind::StackedBar | ChartKind::Line | ChartKind::Area => {
            cartesian::hit(size, frame, at)
        }
        ChartKind::Pie | ChartKind::Donut | ChartKind::Radial => pie::hit(size, frame, at),
        ChartKind::Radar => radar::hit(size, frame, at),
    }
}

/// The name of a kind in the chart's accessible name.
fn kind_name(kind: ChartKind) -> Value<String> {
    match kind {
        ChartKind::Bar => message("basic.chart.bar", "Bar chart"),
        ChartKind::StackedBar => message("basic.chart.stacked_bar", "Stacked bar chart"),
        ChartKind::Line => message("basic.chart.line", "Line chart"),
        ChartKind::Area => message("basic.chart.area", "Area chart"),
        ChartKind::Pie => message("basic.chart.pie", "Pie chart"),
        ChartKind::Donut => message("basic.chart.donut", "Donut chart"),
        ChartKind::Radar => message("basic.chart.radar", "Radar chart"),
        ChartKind::Radial => message("basic.chart.radial", "Radial chart"),
    }
}

/// A chart (chart.tsx). Created by [`chart`].
///
/// The plot is drawn on one canvas in the theme's chart colors, with a hover
/// tooltip card and a legend under it. The chart is a `Role::Image` named by
/// its kind and series, and described by its data as text, so assistive
/// technology reads the values the picture shows.
pub struct Chart {
    root: Node,
    config: ChartConfig,
    data: Value<Vec<ChartRow>>,
    options: Options,
    format: Rc<dyn Fn(f64) -> String>,
    tooltip: Value<bool>,
    legend: Value<bool>,
    height: Value<Length>,
}

/// A chart of `data` with the series of `config`.
///
/// Every row gives one value per series. The chart is a bar chart with a
/// grid, axes, a tooltip and a legend until [`Chart::kind`] and the other
/// options say otherwise. It is as wide as its container; set the width with
/// `.style(..)`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn visitors() -> impl IntoView {
///     let config = basic::ChartConfig::new([
///         basic::ChartSeries::new("desktop", "Desktop"),
///         basic::ChartSeries::new("mobile", "Mobile"),
///     ]);
///     let data = vec![
///         basic::ChartRow::new("January", [186.0, 80.0]),
///         basic::ChartRow::new("February", [305.0, 200.0]),
///     ];
///     basic::chart(config, data).kind(basic::ChartKind::Line)
/// }
/// ```
#[track_caller]
pub fn chart(config: ChartConfig, data: impl IntoValue<Vec<ChartRow>>) -> Chart {
    Chart {
        root: column(()).style(style().width(pct(100)).min_width(0)),
        config,
        data: data.into_value(),
        options: Options {
            kind: ChartKind::Bar,
            horizontal: false,
            grid: true.into_value(),
            axis: true.into_value(),
        },
        format: Rc::new(format_value),
        tooltip: true.into_value(),
        legend: true.into_value(),
        height: Length::from(240).into_value(),
    }
}

impl Chart {
    /// Choose how the data is drawn.
    pub fn kind(mut self, kind: ChartKind) -> Self {
        self.options.kind = kind;
        self
    }

    /// Lay bars out horizontally with the categories down the left
    /// ([`Orientation::Horizontal`]) or as columns ([`Orientation::Vertical`]
    /// and the default `Unspecified`). It affects bar and stacked bar charts
    /// only. A constant, like the kind: it is the shape of the chart.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.options.horizontal = orientation == Orientation::Horizontal;
        self
    }

    /// Write the values the chart shows (axis ticks, the donut total, the
    /// tooltip and the description for assistive technology) with `format`.
    /// The default groups thousands and keeps at most three decimals
    /// (`1,234.5`).
    pub fn value_format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.format = Rc::new(format);
        self
    }

    /// Show the tooltip card while the pointer is over the plot (default on;
    /// constant or reactive).
    #[track_caller]
    pub fn tooltip(mut self, tooltip: impl IntoValue<bool>) -> Self {
        self.tooltip = tooltip.into_value();
        self
    }

    /// Show the legend under the plot (default on; constant or reactive).
    #[track_caller]
    pub fn legend(mut self, legend: impl IntoValue<bool>) -> Self {
        self.legend = legend.into_value();
        self
    }

    /// Draw the grid lines (default on; constant or reactive).
    #[track_caller]
    pub fn grid(mut self, grid: impl IntoValue<bool>) -> Self {
        self.options.grid = grid.into_value();
        self
    }

    /// Draw the axis labels (default on; constant or reactive).
    #[track_caller]
    pub fn axis(mut self, axis: impl IntoValue<bool>) -> Self {
        self.options.axis = axis.into_value();
        self
    }

    /// Set the height of the plot (default 240 logical pixels), constant or
    /// reactive.
    #[track_caller]
    pub fn height(mut self, height: impl IntoValue<Length>) -> Self {
        self.height = height.into_value();
        self
    }
}

impl NodeBuilder for Chart {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// The data as text, row by row: `January: Desktop 186, Mobile 80; February: ...`.
fn describe(config: &ChartConfig, rows: &[ChartRow], format: &dyn Fn(f64) -> String) -> String {
    rows.iter()
        .map(|row| {
            let values = config
                .series
                .iter()
                .enumerate()
                .map(|(i, series)| {
                    let value = row.values.get(i).copied().unwrap_or(0.0);
                    format!("{} {}", series.label, format(value))
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}: {values}", row.label)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

impl IntoView for Chart {
    fn into_view(self) -> View {
        let Chart {
            root,
            config,
            data,
            options,
            format,
            tooltip: with_tooltip,
            legend: with_legend,
            height,
        } = self;
        let config = Rc::new(config);
        let hit = signal(None::<Hit>);
        // Apart from the hit: a move within a row only moves the card.
        let at = signal(Point::new(0.0, 0.0));
        let plot = Rc::new(Cell::new(Size::new(0.0, 0.0)));
        let total_label = message("basic.chart.total", "Total");
        let kind = kind_name(options.kind);
        let series_names = config
            .series
            .iter()
            .map(|series| series.label.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let name = message_with("basic.chart.summary", "{kind}: {series}", move || {
            [("kind", kind.get()), ("series", series_names.clone())]
        });
        let details = {
            let (config, format) = (config.clone(), format.clone());
            memo(move || data.with(|rows| describe(&config, rows, &*format)))
        };
        let canvas = node()
            .role(Role::Image)
            .name(name)
            .description(details)
            .style(move || style().width(pct(100)).height(height.get()).shrink(0))
            .draw({
                let (config, plot, format) = (config.clone(), plot.clone(), format.clone());
                move |d, size| {
                    plot.set(size);
                    let label = total_label.get();
                    data.with(|rows| {
                        let frame = options.frame(&config, rows, &label, &*format);
                        paint(d, size, &frame, hit.get().as_ref());
                    });
                }
            })
            .on_pointer({
                let (config, plot, format) = (config.clone(), plot.clone(), format.clone());
                move |event: PointerEvent| {
                    if !with_tooltip.get_untracked() {
                        return;
                    }
                    match event.phase {
                        PointerPhase::Down | PointerPhase::Move | PointerPhase::Enter => {
                            let next = data.with_untracked(|rows| {
                                let frame = options.frame(&config, rows, "", &*format);
                                hit_test(plot.get(), &frame, event.pos)
                            });
                            at.set(event.pos);
                            hit.set(next);
                        }
                        PointerPhase::Leave | PointerPhase::Cancel => hit.set(None),
                        PointerPhase::Up => {}
                    }
                }
            });
        let card = {
            let config = config.clone();
            dynamic(
                move || hit.get(),
                move |hit| hit.map(|hit| tooltip::tooltip(&hit, at, &config, plot.get(), &*format)),
            )
        };
        let legend = {
            let config = config.clone();
            let shown = !config.series.is_empty();
            show(
                move || shown && with_legend.get(),
                move || tooltip::legend(&config),
            )
        };
        let plot_box = node()
            .style(style().width(pct(100)).direction(Direction::Column))
            .child(canvas)
            .child(card);
        root.child(plot_box).child(legend).into_view()
    }
}
