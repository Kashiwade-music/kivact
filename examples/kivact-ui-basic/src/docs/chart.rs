//! Catalog page of Chart: bar, stacked, line, area, pie, radar and radial.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::view::Orientation;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("chart", "Chart", "Components", include_str!("chart.rs"))
        .description("Draws bars, lines, areas, pies, radars and more from the series of a ChartConfig and the rows of ChartRow data. Hovering shows a tooltip with the values. Pass false to axis, grid, legend or tooltip to remove them, and value_format changes how values are written.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "bar",
            "Bar Chart",
            "The default kind is Bar. A series color that is not given comes from the theme's chart 1 to 5, and the legend and the tooltip use the same colors.",
            bar,
        ))
        .example(Example::new(
            "horizontal",
            "Horizontal Stacked",
            "Pass StackedBar as kind and Orientation::Horizontal as orientation to get stacked horizontal bars. value_format writes the axis and the tooltip values in the form '186k'.",
            horizontal,
        ))
        .example(Example::new(
            "line",
            "Line Chart",
            "Pass Line as kind. A series color can be set with ChartSeries::color.",
            line,
        ))
        .example(Example::new(
            "area",
            "Minimal Area Chart",
            "Area fills the area under the line with a translucent color. Here axis, grid and legend are all false, so only the area and the tooltip remain.",
            area,
        ))
        .example(Example::new(
            "pie",
            "Pie and Donut",
            "Pie and Donut split the circle by the total of each series. Donut shows the grand total in its center.",
            pie,
        ))
        .example(Example::new(
            "radar",
            "Radar and Radial",
            "Radar overlays a polygon of each series on the axes of each row. Radial draws an arc per series, and the largest series spans 270 degrees.",
            radar,
        ))
}

// example: bar
fn bar() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
        basic::ChartRow::new("Apr", [73.0, 190.0]),
        basic::ChartRow::new("May", [209.0, 130.0]),
        basic::ChartRow::new("Jun", [214.0, 140.0]),
    ];
    kivact::ui::card(basic::chart(config, data))
        .header(
            column((
                text("Bar Chart").style(
                    style()
                        .font_size(14)
                        .line_height(20.0 / 14.0)
                        .font_weight(500),
                ),
                text("January - June 2024")
                    .style(style().line_height(1.625).color(basic::MUTED_FOREGROUND)),
            ))
            .style(style().gap(4).flex_grow(1).min_width(0)),
        )
        .footer(text("Showing total visitors for the last 6 months"))
        .style(style().width(560))
}
// end example

// example: horizontal
fn horizontal() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
        basic::ChartRow::new("Apr", [73.0, 190.0]),
    ];
    let chart = basic::chart(config, data)
        .kind(basic::ChartKind::StackedBar)
        .orientation(Orientation::Horizontal)
        .value_format(|value| format!("{value:.0}k"))
        .height(200.0);
    kivact::ui::card(chart)
        .header(
            column((
                text("Horizontal Stacked").style(
                    style()
                        .font_size(14)
                        .line_height(20.0 / 14.0)
                        .font_weight(500),
                ),
                text("Visitors by device")
                    .style(style().line_height(1.625).color(basic::MUTED_FOREGROUND)),
            ))
            .style(style().gap(4).flex_grow(1).min_width(0)),
        )
        .style(style().width(560))
}
// end example

// example: line
fn line() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop").color(rgb(0x2563eb)),
        basic::ChartSeries::new("mobile", "Mobile").color(rgb(0xe11d48)),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
        basic::ChartRow::new("Apr", [73.0, 190.0]),
        basic::ChartRow::new("May", [209.0, 130.0]),
        basic::ChartRow::new("Jun", [214.0, 140.0]),
    ];
    kivact::ui::card(basic::chart(config, data).kind(basic::ChartKind::Line))
        .header(
            column((
                text("Line Chart").style(
                    style()
                        .font_size(14)
                        .line_height(20.0 / 14.0)
                        .font_weight(500),
                ),
                text("January - June 2024")
                    .style(style().line_height(1.625).color(basic::MUTED_FOREGROUND)),
            ))
            .style(style().gap(4).flex_grow(1).min_width(0)),
        )
        .style(style().width(560))
}
// end example

// example: area
fn area() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
        basic::ChartRow::new("Apr", [73.0, 190.0]),
        basic::ChartRow::new("May", [209.0, 130.0]),
        basic::ChartRow::new("Jun", [214.0, 140.0]),
    ];
    let chart = basic::chart(config, data)
        .kind(basic::ChartKind::Area)
        .axis(false)
        .grid(false)
        .legend(false)
        .height(160.0);
    kivact::ui::card(chart)
        .header(
            column((
                text("Area Chart").style(
                    style()
                        .font_size(14)
                        .line_height(20.0 / 14.0)
                        .font_weight(500),
                ),
                text("Last 6 months")
                    .style(style().line_height(1.625).color(basic::MUTED_FOREGROUND)),
            ))
            .style(style().gap(4).flex_grow(1).min_width(0)),
        )
        .style(style().width(560))
}
// end example

// example: pie
fn pie() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("chrome", "Chrome"),
        basic::ChartSeries::new("safari", "Safari"),
        basic::ChartSeries::new("firefox", "Firefox"),
        basic::ChartSeries::new("edge", "Edge"),
    ]);
    let data = vec![basic::ChartRow::new(
        "Visitors",
        [275.0, 200.0, 187.0, 173.0],
    )];
    let pie = |kind| {
        basic::chart(config.clone(), data.clone())
            .kind(kind)
            .axis(false)
            .height(200.0)
            .style(style().width(240))
    };
    row((pie(basic::ChartKind::Pie), pie(basic::ChartKind::Donut))).style(style().gap(24))
}
// end example

// example: radar
fn radar() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("January", [186.0, 80.0]),
        basic::ChartRow::new("February", [305.0, 200.0]),
        basic::ChartRow::new("March", [237.0, 120.0]),
        basic::ChartRow::new("April", [73.0, 190.0]),
        basic::ChartRow::new("May", [209.0, 130.0]),
        basic::ChartRow::new("June", [214.0, 140.0]),
    ];
    let chart = |kind| {
        basic::chart(config.clone(), data.clone())
            .kind(kind)
            .height(240.0)
            .style(style().width(270))
    };
    row((
        chart(basic::ChartKind::Radar),
        chart(basic::ChartKind::Radial),
    ))
    .style(style().gap(16))
}
// end example
