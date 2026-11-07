//! Role: Catalog page of Plot: lines, points, stems, histograms and heatmaps on zoomable axes.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;
use basic::plot::{
    AxisScale, LegendPosition, LineDash, PlotAxis, PlotColor, PlotData, PlotModel, PlotRange,
    PlotSeries, PlotSeriesKind, TickFormat,
};

pub(crate) fn page() -> Page {
    Page {
        slug: "plot",
        name: "Plot",
        description: "PlotModel の系列を、ズームとパンができる軸の上に描きます。ドラッグで移動、ホイールで x 軸を拡大・縮小（Ctrl で y 軸、Shift で移動）、ダブルクリックで自動の範囲に戻ります。軸は線形か対数（Log10）を選べ、目盛りの書式も変えられます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic::plot::{PlotModel, PlotRange, plot};",
        source: include_str!("plot.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "frequency",
                "Frequency Response",
                "x 軸を対数、目盛りを SI の書式にした折れ線です。範囲は PlotRange::Auto から始まり、操作のたびに PlotRange::Manual へ書き込まれます。",
                frequency,
            ),
            Example::new(
                "heatmap",
                "Heatmap",
                "PlotSeriesKind::Heatmap は、行と列の境界（edges）と z の値の格子を色で描きます。ポインタを重ねると、そのセルの値を読めます。",
                heatmap,
            ),
        ],
    }
}

// example: frequency
fn frequency() -> impl IntoView {
    let frequencies: Vec<f64> = (0..=60)
        .map(|i| 10f64.powf(1.0 + f64::from(i) / 20.0))
        .collect();
    let gain: Vec<f64> = frequencies
        .iter()
        .map(|f| 20.0 * (1.0 / (1.0 + (f / 200.0).powi(2)).sqrt()).log10())
        .collect();
    let model = PlotModel {
        title: "Low-pass response".into(),
        x: PlotAxis {
            label: "Frequency".into(),
            scale: AxisScale::Log10,
            format: TickFormat::Si,
        },
        y: PlotAxis {
            label: "Gain".into(),
            scale: AxisScale::Linear,
            format: TickFormat::Decibel,
        },
        series: vec![PlotSeries {
            name: "Gain".into(),
            color: PlotColor::Palette(0),
            width: 2.0,
            dash: LineDash::Solid,
            kind: PlotSeriesKind::Line {
                x: PlotData::new(frequencies),
                y: PlotData::new(gain),
            },
        }],
        legend: LegendPosition::Hidden,
    };
    let range = signal(PlotRange::Auto);
    basic::plot::plot("Low-pass response", model, range).style(style().width(560))
}
// end example

// example: heatmap
fn heatmap() -> impl IntoView {
    let (rows, cols) = (12_u32, 24_u32);
    let x_edges: Vec<f64> = (0..=cols).map(f64::from).collect();
    let y_edges: Vec<f64> = (0..=rows).map(f64::from).collect();
    let z: Vec<f64> = (0..rows)
        .flat_map(|r| {
            (0..cols).map(move |c| {
                let (r, c) = (f64::from(r), f64::from(c));
                (r * 0.5).sin() + (c * 0.3).cos()
            })
        })
        .collect();
    let model = PlotModel {
        title: "Cell values".into(),
        x: PlotAxis {
            label: "Column".into(),
            scale: AxisScale::Linear,
            format: TickFormat::Plain,
        },
        y: PlotAxis {
            label: "Row".into(),
            scale: AxisScale::Linear,
            format: TickFormat::Plain,
        },
        series: vec![PlotSeries {
            name: "Value".into(),
            color: PlotColor::Palette(0),
            width: 1.0,
            dash: LineDash::Solid,
            kind: PlotSeriesKind::Heatmap {
                x_edges: PlotData::new(x_edges),
                y_edges: PlotData::new(y_edges),
                z: PlotData::new(z),
                rows,
                cols,
                z_min: -2.0,
                z_max: 2.0,
            },
        }],
        legend: LegendPosition::Hidden,
    };
    basic::plot::plot("Cell values", model, signal(PlotRange::Auto)).style(style().width(560))
}
// end example
