//! Catalog page of Plot: lines, points, stems, histograms and heatmaps.

use catalog_shell::{Example, Page};
use kivact::prelude::*;

pub(crate) fn page() -> Page {
    Page::new("plot", "Plot", "Components", include_str!("plot.rs"))
        .description("Draws the series of a PlotModel on axes that can be zoomed and panned. Drag to move; the wheel zooms the x axis (with Ctrl, the y axis; the horizontal wheel and the wheel with Shift move it); + and -, PageUp, PageDown and the arrow keys work too; 0 or a double click returns to the automatic range. Each axis can be linear or logarithmic (Log10), and the tick format can be changed.")
        .usage("use kivact::prelude::*;\nuse kivact_plot::{PlotAxis, PlotModel, PlotSeries, plot};")
        .example(Example::new(
            "frequency",
            "Frequency Response",
            "A line with a logarithmic x axis and SI tick labels. The range starts as PlotRange::Auto, and each change makes it PlotRange::Manual.",
            frequency,
        ))
        .example(Example::new(
            "heatmap",
            "Heatmap",
            "PlotSeries::heatmap draws a grid of z values, with the row and column edges, in colors. Unless z_range sets the color range, the finite minimum and maximum of z are used. Pointing at a cell shows its value.",
            heatmap,
        ))
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
    let model = kivact_plot::PlotModel::new(
        kivact_plot::PlotAxis::new("Frequency")
            .scale(kivact_plot::AxisScale::Log10)
            .format(kivact_plot::TickFormat::Si),
        kivact_plot::PlotAxis::new("Gain").format(kivact_plot::TickFormat::Decibel),
    )
    .title("Low-pass response")
    .legend(false)
    .series(kivact_plot::PlotSeries::line("Gain", frequencies, gain));
    kivact_plot::plot(model).style(style().width(560))
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
    let model = kivact_plot::PlotModel::new(
        kivact_plot::PlotAxis::new("Column"),
        kivact_plot::PlotAxis::new("Row"),
    )
    .title("Cell values")
    .legend(false)
    .series(kivact_plot::PlotSeries::heatmap("Value", x_edges, y_edges, z).z_range(-2.0, 2.0));
    kivact_plot::plot(model).style(style().width(560))
}
// end example
