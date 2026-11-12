//! Role: The Chart builder and its canvas, tooltip, legend and accessible text.

use super::model::Hit;
use super::model::{ChartConfig, ChartKind, ChartRow};
use super::plot::{Options, hit_test, paint};
use kivact::geometry::{Point, Rect, Size};
use kivact::i18n::{FractionDigits, Grouping, Locale, NumberStyle, use_locale};
use kivact::i18n::{message, message_with};
use kivact::state::IntoValue;
use kivact::state::{Signal, Value, memo, signal};
use kivact::style::{Direction, Length, pct, style};
use kivact::view::{Element, Node, column, node, node_ref};
use kivact::view::{IntoView, Orientation, PointerEvent, PointerPhase, Role, View, dynamic, show};
use std::rc::Rc;

/// The name of a kind in the chart's accessible name.
fn kind_name(kind: Value<ChartKind>) -> Value<String> {
    let names = [
        message("basic.chart.bar", "Bar chart"),
        message("basic.chart.stacked_bar", "Stacked bar chart"),
        message("basic.chart.line", "Line chart"),
        message("basic.chart.area", "Area chart"),
        message("basic.chart.pie", "Pie chart"),
        message("basic.chart.donut", "Donut chart"),
        message("basic.chart.radar", "Radar chart"),
        message("basic.chart.radial", "Radial chart"),
    ];
    memo(move || match kind.get() {
        ChartKind::Bar => names[0].get(),
        ChartKind::StackedBar => names[1].get(),
        ChartKind::Line => names[2].get(),
        ChartKind::Area => names[3].get(),
        ChartKind::Pie => names[4].get(),
        ChartKind::Donut => names[5].get(),
        ChartKind::Radar => names[6].get(),
        ChartKind::Radial => names[7].get(),
    })
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
    format: Option<Rc<dyn Fn(f64) -> String>>,
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
            kind: ChartKind::Bar.into_value(),
            horizontal: false.into_value(),
            grid: true.into_value(),
            axis: true.into_value(),
        },
        format: None,
        tooltip: true.into_value(),
        legend: true.into_value(),
        height: Length::from(240).into_value(),
    }
}

impl Chart {
    /// Choose how the data is drawn.
    pub fn kind(mut self, kind: impl IntoValue<ChartKind>) -> Self {
        self.options.kind = kind.into_value();
        self
    }

    /// Lay bars out horizontally with the categories down the left
    /// ([`Orientation::Horizontal`]) or as columns ([`Orientation::Vertical`]
    /// and the default `Unspecified`). It affects bar and stacked bar charts
    /// only. A constant, like the kind: it is the shape of the chart.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        let orientation = orientation.into_value();
        self.options.horizontal =
            (move || orientation.get() == Orientation::Horizontal).into_value();
        self
    }

    /// Write the values the chart shows (axis ticks, the donut total, the
    /// tooltip and the description for assistive technology) with `format`.
    /// The default writes the number in the app's locale, at most three
    /// decimals, with grouping (`1,234.5` in en-US).
    pub fn value_format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
        self.format = Some(Rc::new(format));
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

impl Element for Chart {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// The default value text: the app's locale, at most three decimals, grouped.
/// The locale is read where the text is written, so a locale change draws and
/// describes the chart again.
fn default_format(locale: Value<Locale>) -> Rc<dyn Fn(f64) -> String> {
    Rc::new(move |value: f64| {
        if !value.is_finite() {
            return "-".to_owned();
        }
        // A value that rounds to zero at three decimals is written without its sign.
        let value = if value.abs() < 0.0005 { 0.0 } else { value };
        locale.with(|locale| {
            locale.format_number(
                value,
                NumberStyle {
                    fraction_digits: FractionDigits::Range { min: 0, max: 3 },
                    grouping: Grouping::Locale,
                },
            )
        })
    })
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

/// What the pointer is over: the tooltip's content and where the pointer is.
#[derive(Clone, Debug, PartialEq)]
enum ChartHover {
    /// Not over the plot.
    Away,
    /// Over `hit` at `at` (in the plot).
    Over { hit: Hit, at: Point },
}

/// The hit under the pointer and the pointer position, as two memos of the
/// hover. The hit and the pointer change separately: a move within a row only
/// moves the card.
fn hover_values(hover: Signal<ChartHover>) -> (Value<Option<Hit>>, Value<Point>) {
    let hit = memo(move || {
        hover.with(|hover| match hover {
            ChartHover::Over { hit, .. } => Some(hit.clone()),
            ChartHover::Away => None,
        })
    });
    let at = memo(move || {
        hover.with(|hover| match hover {
            ChartHover::Over { at, .. } => *at,
            ChartHover::Away => Point::new(0.0, 0.0),
        })
    });
    (hit, at)
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
        let locale = use_locale();
        let format = format.unwrap_or_else(|| default_format(locale));
        let hover = signal(ChartHover::Away);
        let (hit, at) = hover_values(hover);
        let target = node_ref();
        let bounds = target.rect();
        let total_label = message("basic.chart.total", "Total");
        let name = chart_name(options.kind, &config);
        let details = {
            let (config, format) = (config.clone(), format.clone());
            memo(move || data.with(|rows| describe(&config, rows, &*format)))
        };
        let canvas = node()
            .with_node(|root| root.role(Role::Image))
            .name(name)
            .description(details)
            .style(move || style().width(pct(100)).height(height.get()).shrink(0))
            .node_ref(target)
            .draw({
                let (config, format) = (config.clone(), format.clone());
                move |d, size| {
                    let label = total_label.get();
                    data.with(|rows| {
                        let frame = options.frame(&config, rows, &label, &*format);
                        hit.with(|hit| paint(d, size, &frame, hit.as_ref()));
                    });
                }
            })
            .on_pointer({
                let (config, format) = (config.clone(), format.clone());
                move |event: PointerEvent| {
                    if !with_tooltip.get_untracked() {
                        return;
                    }
                    match event.phase() {
                        PointerPhase::Down(_) | PointerPhase::Move | PointerPhase::Enter => {
                            let size = bounds.get_untracked().map_or(Size::new(0.0, 0.0), |rect| {
                                Size::new(rect.width, rect.height)
                            });
                            let next = data.with_untracked(|rows| {
                                let frame = options.frame(&config, rows, "", &*format);
                                hit_test(size, &frame, event.pos())
                            });
                            hover.set(match next {
                                Some(hit) => ChartHover::Over {
                                    hit,
                                    at: event.pos(),
                                },
                                None => ChartHover::Away,
                            });
                        }
                        PointerPhase::Leave | PointerPhase::Cancel => hover.set(ChartHover::Away),
                        PointerPhase::Up(_) => {}
                    }
                }
            });
        let (card, legend) = below_plot(&config, bounds, &format, hit, at, with_legend, locale);
        let plot_box = node()
            .style(style().width(pct(100)).direction(Direction::Column))
            .child(canvas)
            .child(card);
        root.child(plot_box).child(legend).into_view()
    }
}

/// The tooltip card that follows the hit and the legend under the plot.
fn below_plot(
    config: &Rc<ChartConfig>,
    bounds: Value<Option<Rect>>,
    format: &Rc<dyn Fn(f64) -> String>,
    hit: Value<Option<Hit>>,
    at: Value<Point>,
    with_legend: Value<bool>,
    locale: Value<Locale>,
) -> (impl IntoView, impl IntoView) {
    let card_config = config.clone();
    let format = format.clone();
    // The card's text is written in the locale, so a locale change rebuilds it.
    let card = dynamic(
        move || (hit.get(), locale.get()),
        move |(hit, _locale)| {
            hit.map(|hit| super::tooltip::tooltip(&hit, at, &card_config, bounds, &*format))
        },
    );
    let legend_config = config.clone();
    let shown = !config.series.is_empty();
    let legend = show(
        move || shown && with_legend.get(),
        move || super::tooltip::legend(&legend_config),
    );
    (card, legend)
}

fn chart_name(kind: Value<ChartKind>, config: &ChartConfig) -> Value<String> {
    let kind = kind_name(kind);
    let series_names = config
        .series
        .iter()
        .map(|series| series.label.clone())
        .collect::<Vec<_>>()
        .join(", ");
    message_with("basic.chart.summary", "{kind}: {series}", move || {
        [("kind", kind.get()), ("series", series_names.clone())]
    })
}
