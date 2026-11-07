//! Role: Paints a plot: grid, axes, labels, series, legend and the hover readout.

use super::decimate::{Bounds, runs};
use super::heatmap::{Cells, paint as paint_heatmap};
use super::input::{maps, readout};
use super::model::AxisScale;
use super::model::{
    LegendPosition, LineDash, PlotColor, PlotModel, PlotSeries, PlotSeriesKind, PlotView,
};
use super::scale::Map;
use super::ticks::{LogTick, format_value, linear_ticks, log_ticks};
use crate::tokens::Tokens;
use kivact::draw::{Draw, Layer, LineCap, Stroke, TextBaseline, TextFormat};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::{Color, TextAlign, rgb};

/// The space the axes and title take around the plot, in logical px.
const LEFT: f64 = 64.0;
const RIGHT: f64 = 16.0;
const BOTTOM: f64 = 40.0;
const TOP_WITH_TITLE: f64 = 24.0;
const TOP: f64 = 8.0;
/// The distance between two tick labels (about), in logical px.
const X_TICK_PX: f64 = 80.0;
const Y_TICK_PX: f64 = 50.0;
/// The size of the text.
const TEXT: f64 = 12.0;

/// The plot rectangle inside `size` for `model`: the bounds the series and
/// the readout are drawn in. Empty (right ≤ left or bottom ≤ top) when the
/// size leaves no room.
pub(super) fn layout(size: Size, model: &PlotModel) -> Bounds {
    let top = if model.title.is_empty() {
        TOP
    } else {
        TOP_WITH_TITLE
    };
    Bounds {
        left: LEFT,
        top,
        right: size.width - RIGHT,
        bottom: size.height - BOTTOM,
    }
}

/// Paint `model` at `view` over a canvas of `size` (logical px, the theme's
/// chart colors and plot tokens), with no pointer.
///
/// The plot is drawn in the rectangle [`layout`] leaves: its background, grid
/// lines, axes, tick labels, axis titles, the title, the series and the
/// legend in the top right corner.
pub fn draw_plot(draw: &mut Draw, size: Size, model: &PlotModel, view: PlotView) {
    paint(draw, size, model, view, None);
}

/// [`draw_plot`] with a pointer at `hover` (plot pixels): a crosshair, a
/// marker on the point nearest to it and a readout card.
pub(super) fn paint(
    draw: &mut Draw,
    size: Size,
    model: &PlotModel,
    view: PlotView,
    hover: Option<(f64, f64)>,
) {
    let bounds = layout(size, model);
    if bounds.right <= bounds.left || bounds.bottom <= bounds.top {
        return;
    }
    let tokens = draw.theme().ext::<Tokens>();
    let (mx, my) = maps(model, view, bounds);
    let plot = Rect::new(
        bounds.left,
        bounds.top,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
    );
    draw.fill_rect(plot, tokens.plot.background);
    let x_ticks = ticks(
        model.x.scale,
        view.x_min,
        view.x_max,
        plot.width / X_TICK_PX,
    );
    let y_ticks = ticks(
        model.y.scale,
        view.y_min,
        view.y_max,
        plot.height / Y_TICK_PX,
    );
    paint_grid(draw, bounds, &mx, &my, &x_ticks, &y_ticks, &tokens);
    paint_axes(draw, model, bounds, &mx, &my, &x_ticks, &y_ticks, &tokens);
    if !model.title.is_empty() {
        let style = text_style(
            TEXT + 2.0,
            tokens.plot.text,
            TextAlign::Center,
            TextBaseline::Top,
        );
        draw.text(&model.title, Point::new(size.width / 2.0, 4.0), style);
    }
    draw.layer(Layer::new().clip_rect(plot), |draw| {
        let stops: Vec<[f64; 4]> = tokens
            .plot
            .colormap
            .iter()
            .map(|c| {
                let c = c.resolve(draw.theme());
                [c.r, c.g, c.b, c.a]
            })
            .collect();
        for series in &model.series {
            let color = series_color(series.color, &tokens);
            paint_series(draw, series, color, bounds, &mx, &my, view, &stops);
        }
    });
    if model.legend == LegendPosition::TopRight && !model.series.is_empty() {
        paint_legend(draw, model, bounds, &tokens);
    }
    if let Some((px, py)) = hover {
        let inside =
            px >= bounds.left && px <= bounds.right && py >= bounds.top && py <= bounds.bottom;
        if inside {
            paint_hover(draw, model, view, bounds, (px, py), &tokens);
        }
    }
}

/// The ticks of one axis: linear ticks, or log ticks with their major flag.
fn ticks(scale: AxisScale, lo: f64, hi: f64, target: f64) -> Vec<LogTick> {
    let target = target.floor().max(1.0) as usize;
    match scale {
        AxisScale::Linear => linear_ticks(lo, hi, target)
            .into_iter()
            .map(|value| LogTick { value, major: true })
            .collect(),
        AxisScale::Log10 => log_ticks(lo, hi, target),
    }
}

/// The spacing of the linear ticks, which decides the decimals of their labels.
fn spacing(ticks: &[LogTick], scale: AxisScale) -> f64 {
    match (scale, ticks) {
        (AxisScale::Linear, [a, b, ..]) => (b.value - a.value).abs(),
        _ => 0.0,
    }
}

fn text_style(size: f64, color: Color, align: TextAlign, baseline: TextBaseline) -> TextFormat {
    TextFormat {
        size,
        color,
        align,
        baseline,
        ..TextFormat::default()
    }
}

fn paint_grid(
    draw: &mut Draw,
    bounds: Bounds,
    mx: &Map,
    my: &Map,
    x_ticks: &[LogTick],
    y_ticks: &[LogTick],
    tokens: &Tokens,
) {
    for tick in x_ticks {
        if let Some(x) = mx.px(tick.value) {
            let (color, width) = grid_style(tick.major, tokens);
            draw.line(
                Point::new(x, bounds.top),
                Point::new(x, bounds.bottom),
                width,
                color,
            );
        }
    }
    for tick in y_ticks {
        if let Some(y) = my.px(tick.value) {
            let (color, width) = grid_style(tick.major, tokens);
            draw.line(
                Point::new(bounds.left, y),
                Point::new(bounds.right, y),
                width,
                color,
            );
        }
    }
}

fn grid_style(major: bool, tokens: &Tokens) -> (Color, f64) {
    if major {
        (tokens.plot.grid, 1.0)
    } else {
        (tokens.plot.grid_minor, 0.5)
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "the axes take the model, both maps, both tick sets and the tokens"
)]
fn paint_axes(
    draw: &mut Draw,
    model: &PlotModel,
    bounds: Bounds,
    mx: &Map,
    my: &Map,
    x_ticks: &[LogTick],
    y_ticks: &[LogTick],
    tokens: &Tokens,
) {
    let axis = Stroke::new(1.0);
    draw.line(
        Point::new(bounds.left, bounds.bottom),
        Point::new(bounds.right, bounds.bottom),
        axis.clone(),
        tokens.plot.axis,
    );
    draw.line(
        Point::new(bounds.left, bounds.top),
        Point::new(bounds.left, bounds.bottom),
        axis,
        tokens.plot.axis,
    );
    let x_step = spacing(x_ticks, model.x.scale);
    let y_step = spacing(y_ticks, model.y.scale);
    let label = text_style(TEXT, tokens.plot.text, TextAlign::Center, TextBaseline::Top);
    for tick in x_ticks.iter().filter(|t| t.major) {
        if let Some(x) = mx.px(tick.value) {
            let text = format_value(tick.value, &model.x.format, x_step);
            draw.text(&text, Point::new(x, bounds.bottom + 6.0), label.clone());
        }
    }
    let label = text_style(TEXT, tokens.plot.text, TextAlign::End, TextBaseline::Middle);
    for tick in y_ticks.iter().filter(|t| t.major) {
        if let Some(y) = my.px(tick.value) {
            let text = format_value(tick.value, &model.y.format, y_step);
            draw.text(&text, Point::new(bounds.left - 6.0, y), label.clone());
        }
    }
    let title = text_style(TEXT, tokens.plot.text, TextAlign::Center, TextBaseline::Top);
    if !model.x.label.is_empty() {
        let at = Point::new((bounds.left + bounds.right) / 2.0, bounds.bottom + 22.0);
        draw.text(&model.x.label, at, title.clone());
    }
    if !model.y.label.is_empty() {
        let at = Point::new(bounds.left + 4.0, bounds.top + 4.0);
        let start = text_style(TEXT, tokens.plot.text, TextAlign::Start, TextBaseline::Top);
        draw.text(&model.y.label, at, start);
    }
}

/// The color of a series: a chart color, or a fixed RGB value.
fn series_color(color: PlotColor, tokens: &Tokens) -> Color {
    match color {
        PlotColor::Palette(i) => tokens.chart[i as usize % tokens.chart.len()],
        PlotColor::Rgb(value) => rgb(value),
    }
}

/// The stroke of a line or stem: its width, its dash pattern and caps.
fn stroke_of(series: &PlotSeries) -> Stroke {
    let base = Stroke::new(series.width);
    match series.dash {
        LineDash::Solid => base,
        LineDash::Dashed => Stroke {
            dash: vec![6.0, 4.0],
            ..base
        },
        LineDash::Dotted => Stroke {
            cap: LineCap::Round,
            dash: vec![0.0, 4.0],
            ..base
        },
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "one series is painted from its model, its maps and its color"
)]
fn paint_series(
    draw: &mut Draw,
    series: &PlotSeries,
    color: Color,
    bounds: Bounds,
    mx: &Map,
    my: &Map,
    view: PlotView,
    stops: &[[f64; 4]],
) {
    let base_y = my.px(view.y_min).unwrap_or(bounds.bottom);
    match &series.kind {
        PlotSeriesKind::Line { x, y } => {
            let points: Vec<Option<(f64, f64)>> = x
                .as_slice()
                .iter()
                .zip(y.as_slice())
                .map(|(xv, yv)| mx.px(*xv).zip(my.px(*yv)))
                .collect();
            let stroke = stroke_of(series);
            for run in runs(&points, bounds) {
                draw.polyline(
                    run.into_iter().map(|(x, y)| Point::new(x, y)),
                    &stroke,
                    color,
                );
            }
        }
        PlotSeriesKind::Points { x, y, radius } => {
            for (xv, yv) in x.as_slice().iter().zip(y.as_slice()) {
                if let Some((px, py)) = mx.px(*xv).zip(my.px(*yv)) {
                    draw.circle(Point::new(px, py), *radius, color);
                }
            }
        }
        PlotSeriesKind::Stems { x, y } => {
            let stroke = stroke_of(series);
            for (xv, yv) in x.as_slice().iter().zip(y.as_slice()) {
                if let Some((px, py)) = mx.px(*xv).zip(my.px(*yv)) {
                    draw.line(
                        Point::new(px, base_y),
                        Point::new(px, py),
                        stroke.clone(),
                        color,
                    );
                    draw.circle(Point::new(px, py), 3.0, color);
                }
            }
        }
        PlotSeriesKind::Steps { edges, values } => {
            let (edges, values) = (edges.as_slice(), values.as_slice());
            for (i, height) in values.iter().enumerate() {
                let (Some(a), Some(b), Some(top)) = (
                    edges.get(i).and_then(|e| mx.px(*e)),
                    edges.get(i + 1).and_then(|e| mx.px(*e)),
                    my.px(*height),
                ) else {
                    continue;
                };
                let rect = Rect::new(
                    a.min(b),
                    top.min(base_y),
                    (a - b).abs(),
                    (base_y - top).abs(),
                );
                draw.fill_rect(rect, color);
            }
        }
        PlotSeriesKind::Heatmap {
            x_edges,
            y_edges,
            z,
            rows,
            cols,
            z_min,
            z_max,
        } => {
            let cells = Cells {
                x_edges: x_edges.as_slice(),
                y_edges: y_edges.as_slice(),
                z: z.as_slice(),
                rows: *rows as usize,
                cols: *cols as usize,
                z_min: *z_min,
                z_max: *z_max,
            };
            let rect = Rect::new(
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
            );
            paint_heatmap(draw, rect, mx, my, &cells, stops);
        }
    }
}

fn paint_legend(draw: &mut Draw, model: &PlotModel, bounds: Bounds, tokens: &Tokens) {
    let x = bounds.right - 120.0;
    for (i, series) in model.series.iter().enumerate() {
        let y = bounds.top + 8.0 + i as f64 * 16.0;
        draw.fill_rect(
            Rect::new(x, y + 2.0, 10.0, 10.0),
            series_color(series.color, tokens),
        );
        let style = text_style(TEXT, tokens.plot.text, TextAlign::Start, TextBaseline::Top);
        draw.text(&series.name, Point::new(x + 16.0, y), style);
    }
}

fn paint_hover(
    draw: &mut Draw,
    model: &PlotModel,
    view: PlotView,
    bounds: Bounds,
    (px, py): (f64, f64),
    tokens: &Tokens,
) {
    let line = Stroke::new(1.0);
    draw.line(
        Point::new(px, bounds.top),
        Point::new(px, bounds.bottom),
        line.clone(),
        tokens.plot.crosshair,
    );
    draw.line(
        Point::new(bounds.left, py),
        Point::new(bounds.right, py),
        line,
        tokens.plot.crosshair,
    );
    let Some(read) = readout(model, view, bounds, (px, py)) else {
        return;
    };
    let (sx, sy) = read.at;
    draw.circle(Point::new(sx, sy), 4.0, tokens.plot.crosshair);
    let mut lines = vec![
        read.name.clone(),
        format!("x: {}", format_value(read.x, &model.x.format, 0.0)),
        format!("y: {}", format_value(read.y, &model.y.format, 0.0)),
    ];
    if let Some(z) = read.z {
        lines.push(format!("value: {}", format_value(z, &model.y.format, 0.0)));
    }
    let style = text_style(TEXT, tokens.plot.text, TextAlign::Start, TextBaseline::Top);
    let width = lines
        .iter()
        .map(|text| draw.measure_text(text, &style).size.width)
        .fold(0.0, f64::max);
    let height = 16.0 * lines.len() as f64 + 8.0;
    let card_width = width + 16.0;
    let mut x = px + 12.0;
    if x + card_width > bounds.right {
        x = px - 12.0 - card_width;
    }
    let y = (py + 12.0).min(bounds.bottom - height).max(bounds.top);
    draw.fill_rect(Rect::new(x, y, card_width, height), tokens.popover);
    for (i, text) in lines.iter().enumerate() {
        draw.text(
            text,
            Point::new(x + 8.0, y + 4.0 + 16.0 * i as f64),
            style.clone(),
        );
    }
}
