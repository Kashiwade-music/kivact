# Plot data on zoomable axes

[日本語](../../../jp/basic/how-to-guide/plot.md)

This guide shows how to draw numeric data as an XY plot with `kivact_ui_basic`: a `PlotModel` that describes the title, the axes and the series, a `plot` that the user can zoom, pan and read, and the range that the user edits. It assumes you have done the [tutorial](../getting-started/README.md).

The plot never owns your numbers. A series takes its data as a `Vec<f64>` or an array, and the plot shares it by reference, so cloning it is cheap. A plot is redrawn only when the model or the range changes.

## Draw a line

Build a `PlotModel` from two axes and add a `PlotSeries`. Pass it to `plot` with a label. The plot follows its own range until you bind one (see below).

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::{AxisScale, PlotAxis, PlotModel, PlotSeries, TickFormat, plot};

fn response() -> impl IntoView {
    let frequencies: Vec<f64> = (1..=100).map(|i| f64::from(i) * 10.0).collect();
    let gain: Vec<f64> = frequencies
        .iter()
        .map(|f| -20.0 * (f / 500.0).log10().max(0.0))
        .collect();
    let model = PlotModel::new(
        PlotAxis::new("Frequency")
            .scale(AxisScale::Log10)
            .format(TickFormat::Si),
        PlotAxis::new("Gain").format(TickFormat::Decibel),
    )
    .title("Response")
    .series(PlotSeries::line("Gain", frequencies, gain).color(rgb(0xf2aa4c)));
    plot("Response", model).style(style().width(560))
}
```

- Each kind of series has its own constructor: `PlotSeries::line` (a polyline), `points` (circles), `stems` (a line from the bottom of the plot to each point), `steps` (a histogram from edges and values) and `heatmap` (a grid of cells from their edges and a row-major `z`). Each takes a name, then its data.
- The data are `impl Into<PlotData>`: a `Vec<f64>`, an array such as `[1.0, 2.0]`, or a slice (`&[f64]`, copied). Use `.collect()` for an iterator.
- The setters adjust a series, and each one says which kinds it applies to. `color` applies to every kind. `width` and `dash` apply to lines and stems. `radius` applies to points and to the dots at the top of stems (default 3 px). `z_range` applies to heatmaps.
- A series without `color` takes the kit's chart color of its index, so the series of a plot differ by default.
- The axes are `AxisScale::Linear` or `AxisScale::Log10`. On a log axis, values that are zero or less are not drawn.
- The tick format is `TickFormat::Plain` (four significant digits), `Si` (`1.5k`, `10m`), `Decibel` or `Degrees`. Readouts use the same format. Numbers are written with the digits and the decimal separator of the app's locale.

## Let the user zoom and pan

The plot is a focusable canvas of the image role. Its name is the label, and its description names the series, the axis titles and the visible range, written in the locale.

- Dragging pans. The content follows the pointer.
- The wheel zooms the x axis about the pointer. With Ctrl it zooms the y axis. A horizontal wheel pans x; on most systems Shift with the vertical wheel does the same.
- With the plot focused, `+` and `-` zoom x, PageUp and PageDown zoom y, and the arrow keys move the view by a tenth of its span. `0` and Home return to the automatic range.
- A double click returns to the automatic range.
- Hovering shows a crosshair, a dot on the nearest point and a readout card. A line is read at its nearest x only when its x values ascend.

## Keep the range in your state

Bind a `PlotRange` signal to `range` to read the range the user chose, or to set it from your code. `PlotRange::Auto` fits the data, and `PlotRange::Manual` holds a fixed `PlotView`. Every edit writes the binding at once; there is no separate commit callback.

`on_edit` reports the edits as `EditPhase` values. A drag begins an edit when the pointer first moves it and commits when the button is released. Escape, or a pointer cancel, restores the range the edit started from and reports `Cancel`. A wheel step, a key or a double click is one edit that begins and commits at once.

```rust,no_run
use kivact::prelude::*;
use kivact::view::EditPhase;
use kivact_ui_basic::{PlotModel, PlotRange, plot};

fn watched(model: PlotModel) -> impl IntoView {
    let range = signal(PlotRange::Auto);
    plot("Response", model)
        .range(range)
        .on_edit(move |phase| match phase {
            EditPhase::Begin => {}
            EditPhase::Commit => {
                // `range` holds the new PlotRange::Manual here. Save it, for
                // example to a session file.
            }
            EditPhase::Cancel => {}
        })
}
```

## Draw a heatmap

A heatmap is one series, made by `PlotSeries::heatmap(name, x_edges, y_edges, z)`. Its `x_edges` have one value more than its columns, and its `y_edges` one value more than its rows. The values `z` are row by row, so `z` must hold rows × columns values. A `z` of any other length draws nothing. The rows and columns are derived from the edges, so you give the edges and the values only.

`z_range(min, max)` maps `min` and `max` onto the viridis colormap of the kit's tokens (`Tokens::heatmap`). Without it, the range is the smallest and largest finite value of `z`. A NaN cell is transparent, and a value outside the range takes the nearest end color. The edges may ascend or descend.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic::{PlotAxis, PlotModel, PlotSeries, plot};

fn grid() -> impl IntoView {
    let (rows, cols) = (4_u32, 6_u32);
    let x_edges: Vec<f64> = (0..=cols).map(f64::from).collect();
    let y_edges: Vec<f64> = (0..=rows).map(f64::from).collect();
    let z: Vec<f64> = (0..rows * cols).map(f64::from).collect();
    let model = PlotModel::new(PlotAxis::new("Column"), PlotAxis::new("Row"))
        .legend(false)
        .series(PlotSeries::heatmap("Value", x_edges, y_edges, z).z_range(0.0, 23.0));
    plot("Grid", model).style(style().width(480))
}
```

## Export a plot

`PlotModel::draw` paints the model over a range, with no pointer, into any `Draw`. Pass it the drawing of an `Offscreen` and the same picture goes to an image or to an SVG document. Its colors are the kit's tokens of the theme in effect, and its numbers are written in the locale you pass. `PlotRange::Auto` fits the data; `PlotRange::Manual` shows a fixed `PlotView`.

```rust,no_run
use kivact::draw::Offscreen;
use kivact::i18n::Locale;
use kivact::style::ThemeMode;
use kivact_ui_basic::{PlotModel, PlotRange};

fn export(model: &PlotModel) -> kivact::Result<String> {
    let locale = Locale::new("en-US");
    let mut renderer = Offscreen::new().theme(kivact_ui_basic::theme(ThemeMode::Light));
    renderer.render_svg(640.0, 360.0, |draw, size| {
        model.draw(draw, size, PlotRange::Auto, &locale);
    })
}
```

The same `Offscreen` can also `render` the PNG pixels of this picture, since it keeps one theme and one set of fonts for both. See [Draw with a canvas](../../../../for-kivact-user/en/how-to-guide/canvas.md) for the export of a drawing.

## Keep it readable

- Give every axis a label. A plot without a title does not reserve room for one.
- Keep the number of series small enough for the legend in the top right corner.
- The plot draws about one tick per 80 px horizontally and 50 px vertically. A log axis spanning less than a decade uses the linear ticks of its values.
- The readout card names the x and y values with the axis titles. A heatmap cell's value is labeled by the `basic.plot.value` message (default `Value`), and the accessible description is written in the app's locale.

## See also

- [Display data](data-display.md): tables, charts and badges.
- [Draw with a canvas](../../../../for-kivact-user/en/how-to-guide/canvas.md): the drawing vocabulary the plot uses and the export of a drawing.
