# Plot data on zoomable axes

[日本語](../../../jp/basic/how-to-guide/plot.md)

This guide shows how to draw numeric data as an XY plot with `kivact_ui_basic::plot`: a model that describes the title, the axes and the series, a plot that the user can zoom and pan, and the range that it reports back. It assumes you have done the [tutorial](../getting-started/README.md).

The plot never owns your numbers. A `PlotData` shares its array by reference, so cloning it is cheap, and a plot is redrawn only when the model or the range changes.

## Draw a line

Build a `PlotModel` with one `PlotSeries` and pass it to `plot` with the label and a range. The range is a binding: `PlotRange::Auto` fits the data, and `PlotRange::Manual` fixes it.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::plot::{
    AxisScale, LegendPosition, LineDash, PlotAxis, PlotColor, PlotData, PlotModel, PlotRange,
    PlotSeries, PlotSeriesKind, TickFormat, plot,
};

fn response() -> impl IntoView {
    let frequencies: Vec<f64> = (1..=100).map(|i| f64::from(i) * 10.0).collect();
    let gain: Vec<f64> = frequencies.iter().map(|f| -20.0 * (f / 500.0).log10().max(0.0)).collect();
    let model = PlotModel {
        title: "Response".into(),
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
        legend: LegendPosition::TopRight,
    };
    let range = signal(PlotRange::Auto);
    plot("Response", model, range).style(style().width(560))
}
```

- `PlotSeriesKind` has five kinds: `Line` (a polyline), `Points` (circles of a radius), `Stems` (a line from the bottom of the plot to each point), `Steps` (a histogram from edges and values) and `Heatmap` (a grid of cells from their edges and a row-major `z`).
- A series takes its color from the theme's chart colors (`PlotColor::Palette(i)`) or a fixed `0xRRGGBB` (`PlotColor::Rgb`).
- The axes are linear or `Log10`. On a log axis, values that are zero or less are not drawn.
- The tick format is `Plain` (four significant digits), `Si` (`1.5k`, `10m`), `Decibel` or `Degrees`. Readouts use the same format.

## Let the user zoom and pan

The plot is a canvas named after its label, and its region is named `"<label>: <n> series"` for assistive technology.

- Dragging with the primary button pans. The content follows the pointer.
- The wheel zooms the x axis about the pointer. With Ctrl it zooms the y axis; with Shift it pans x.
- A double click returns the range to `PlotRange::Auto`.
- Hovering shows a crosshair, a marker on the nearest point and a readout card. A line is read at its nearest x only when its x values ascend.

A drag ends at the release, and a run of wheel steps is committed 300 ms after its last step. Each commit writes the new `PlotRange::Manual` to the binding and calls `on_range_commit`, so you can store the range or send it elsewhere:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::plot::{PlotModel, PlotRange, plot};

fn watched(model: PlotModel) -> impl IntoView {
    let range = signal(PlotRange::Auto);
    plot("Response", model, range).on_range_commit(move |committed| {
        // Save `committed`, for example to a session file.
        let _ = committed;
    })
}
```

While a gesture is in progress the plot follows it in the live view, and the binding is not written until the gesture ends.

## Draw a heatmap

A heatmap is one series of `PlotSeriesKind::Heatmap`. Its `x_edges` have `cols + 1` values and its `y_edges` have `rows + 1`. The values `z` are row by row, and `z_min`..`z_max` map onto the viridis colormap of the theme. A NaN cell is transparent, and a cell whose value is outside `z_min`..`z_max` takes the nearest end color. The automatic range fits the edges, and the edges may ascend or descend.

## Export a plot

`draw_plot` paints a model at a view into any `Draw`, with no pointer, so the same picture can go to an `Offscreen` image or an `Svg` document (see [Draw with a canvas](../../../../for-kivact-user/en/how-to-guide/canvas.md)). Use `auto_view` to get the range the data fits.

## Keep it readable

- Give every axis a label; a plot without a title does not reserve room for one.
- Keep the number of series small enough for the legend in the top right corner.
- The plot draws about one tick per 80 px horizontally and 50 px vertically, and a log axis with less than a decade uses the linear ticks of its values.

## See also

- [Display data](data-display.md): tables, charts and badges.
- [Draw with a canvas](../../../../for-kivact-user/en/how-to-guide/canvas.md): the drawing vocabulary the plot uses.
