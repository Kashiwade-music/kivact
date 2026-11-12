//! Role: Catalog page of Plot: lines, points, stems, histograms and heatmaps.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "plot",
        name: "Plot",
        description: "PlotModel の系列を、ズームとパンができる軸の上に描きます。ドラッグで移動、ホイールで x 軸を拡大・縮小（Ctrl で y 軸、横方向のホイールと Shift のホイールで移動）、+ と - と PageUp・PageDown と矢印キーで操作し、0 かダブルクリックで自動の範囲に戻ります。軸は線形か対数（Log10）を選べ、目盛りの書式も変えられます。",
        usage: "use kivact::prelude::*;\nuse kivact_ui_basic as basic;",
        source: include_str!("plot.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "frequency",
                "Frequency Response",
                "x 軸を対数、目盛りを SI の書式にした折れ線です。範囲は PlotRange::Auto から始まり、操作のたびに PlotRange::Manual になります。",
                frequency,
            ),
            Example::new(
                "heatmap",
                "Heatmap",
                "PlotSeries::heatmap は、行と列の境界（edges）と z の値の格子を色で描きます。z_range で色の範囲を決めなければ、z の有限な最小と最大を使います。ポインタを重ねると、そのセルの値を読めます。",
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
    let model = basic::PlotModel::new(
        basic::PlotAxis::new("Frequency")
            .scale(basic::AxisScale::Log10)
            .format(basic::TickFormat::Si),
        basic::PlotAxis::new("Gain").format(basic::TickFormat::Decibel),
    )
    .title("Low-pass response")
    .legend(false)
    .series(basic::PlotSeries::line("Gain", frequencies, gain));
    basic::plot("Low-pass response", model).style(style().width(560))
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
    let model = basic::PlotModel::new(basic::PlotAxis::new("Column"), basic::PlotAxis::new("Row"))
        .title("Cell values")
        .legend(false)
        .series(basic::PlotSeries::heatmap("Value", x_edges, y_edges, z).z_range(-2.0, 2.0));
    basic::plot("Cell values", model).style(style().width(560))
}
// end example
