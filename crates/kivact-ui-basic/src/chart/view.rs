//! Role: The Chart component and its canvas, tooltip, legend and accessible text.

use super::model::{ChartConfig, ChartKind, ChartRow, Hit};
use super::plot::{Options, hit_test, paint};
use kivact::component;
use kivact::geometry::{Point, Rect, Size};
use kivact::i18n::{FractionDigits, Grouping, Locale, NumberStyle, use_locale};
use kivact::i18n::{message, message_with};
use kivact::state::IntoValue;
use kivact::state::{Signal, Value, memo, signal};
use kivact::style::{FlexDirection, Length, pct, style};
use kivact::view::{
    Element, IntoView, Orientation, PointerEvent, PointerPhase, Role, column, dynamic, node,
    node_ref, show,
};
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

/// A chart of `data` with the series of `config` (chart.tsx).
///
/// The plot is drawn on one canvas in the theme's chart colors, with a hover
/// tooltip card and a legend under it. The chart is a `Role::Image` named by
/// its kind and series, and described by its data as text, so assistive
/// technology reads the values the picture shows. Every row gives one value per
/// series. The chart is a bar chart with a grid, axes, a tooltip and a legend
/// until `kind` and the other props say otherwise. It is as wide as its
/// container; set the width with `.style(..)`.
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
#[component]
pub fn chart(
    config: ChartConfig,
    data: Value<Vec<ChartRow>>,
    #[prop(default = ChartKind::Bar)] kind: Value<ChartKind>,
    orientation: Option<Value<Orientation>>,
    value_format: Option<Rc<dyn Fn(f64) -> String>>,
    #[prop(default = true)] tooltip: Value<bool>,
    #[prop(default = true)] legend: Value<bool>,
    #[prop(default = true)] grid: Value<bool>,
    #[prop(default = true)] axis: Value<bool>,
    #[prop(default = Length::from(240))] height: Value<Length>,
) -> impl Element {
    let horizontal = (move || {
        orientation
            .as_ref()
            .is_some_and(|orientation| orientation.get() == Orientation::Horizontal)
    })
    .into_value();
    let options = Options {
        kind,
        horizontal,
        grid,
        axis,
    };
    let config = Rc::new(config);
    let locale = use_locale();
    let format = value_format.unwrap_or_else(|| default_format(locale));
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
        .style(move || {
            style()
                .width(pct(100))
                .height(height.get())
                .flex_shrink(0.0)
        })
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
            let plot = HoverPlot {
                hover,
                bounds,
                data,
                options,
            };
            move |event: PointerEvent| {
                if tooltip.get_untracked() {
                    plot.update(&event, &config, &*format);
                }
            }
        });
    let (card, legend_view) = below_plot(&config, bounds, &format, hit, at, legend, locale);
    let plot_box = node()
        .style(
            style()
                .width(pct(100))
                .flex_direction(FlexDirection::Column),
        )
        .child(canvas)
        .child(card);
    column(())
        .style(style().width(pct(100)).min_width(0))
        .child(plot_box)
        .child(legend_view)
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

/// Where the pointer moves over the plot: the state the hover follows.
#[derive(Clone, Copy)]
struct HoverPlot {
    hover: Signal<ChartHover>,
    bounds: Value<Option<Rect>>,
    data: Value<Vec<ChartRow>>,
    options: Options,
}

impl HoverPlot {
    /// Move the hover to what the pointer of `event` is over: `Down`, `Move`
    /// and `Enter` look up the hit, `Leave` and `Cancel` clear it.
    fn update(self, event: &PointerEvent, config: &ChartConfig, format: &dyn Fn(f64) -> String) {
        match event.phase() {
            PointerPhase::Down(_) | PointerPhase::Move | PointerPhase::Enter => {
                let size = self
                    .bounds
                    .get_untracked()
                    .map_or(Size::new(0.0, 0.0), |rect| {
                        Size::new(rect.width, rect.height)
                    });
                let next = self.data.with_untracked(|rows| {
                    let frame = self.options.frame(config, rows, "", format);
                    hit_test(size, &frame, event.pos())
                });
                self.hover.set(match next {
                    Some(hit) => ChartHover::Over {
                        hit,
                        at: event.pos(),
                    },
                    None => ChartHover::Away,
                });
            }
            PointerPhase::Leave | PointerPhase::Cancel => self.hover.set(ChartHover::Away),
            PointerPhase::Up(_) => {}
        }
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
