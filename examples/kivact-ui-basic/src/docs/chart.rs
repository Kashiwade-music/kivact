//! Role: Catalog page of Chart: bar, stacked, line, area, pie, radar and radial.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "chart",
        name: "Chart",
        description: "ChartConfig の系列と ChartRow のデータから、棒・折れ線・面・円・レーダーなどを描きます。ポインタを重ねると値のツールチップが出ます。軸・グリッド・凡例・ツールチップは axis、grid、legend、tooltip に false を渡して消せ、value_format で値の書き方を変えられます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("chart.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "bar",
                "Bar Chart",
                "既定の kind は Bar です。系列の色は省くとテーマの chart 1〜5 になり、凡例とツールチップにも同じ色が出ます。",
                bar,
            ),
            Example::new(
                "horizontal",
                "Horizontal Stacked",
                "kind に StackedBar、orientation に Orientation::Horizontal を渡すと、積み上げた横棒になります。value_format で軸とツールチップの値を「186k」の形に書きます。",
                horizontal,
            ),
            Example::new(
                "line",
                "Line Chart",
                "kind に Line を渡します。系列の色は ChartSeries::color で指定できます。",
                line,
            ),
            Example::new(
                "area",
                "Minimal Area Chart",
                "Area は折れ線の下を半透明で塗ります。ここでは axis、grid、legend をすべて false にして、面とツールチップだけを残しています。",
                area,
            ),
            Example::new(
                "pie",
                "Pie and Donut",
                "Pie と Donut は系列ごとの合計で円を分けます。Donut は中央に総計を表示します。",
                pie,
            ),
            Example::new(
                "radar",
                "Radar and Radial",
                "Radar は行ごとの軸に系列の多角形を重ねます。Radial は系列ごとの円弧で、最大の系列が 270 度になります。",
                radar,
            ),
        ],
    }
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
    basic::card()
        .header(basic::card_header("Bar Chart").description("January - June 2024"))
        .content(basic::chart(config, data))
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
    basic::card()
        .header(basic::card_header("Horizontal Stacked").description("Visitors by device"))
        .content(chart)
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
    basic::card()
        .header(basic::card_header("Line Chart").description("January - June 2024"))
        .content(basic::chart(config, data).kind(basic::ChartKind::Line))
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
    basic::card()
        .header(basic::card_header("Area Chart").description("Last 6 months"))
        .content(chart)
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
