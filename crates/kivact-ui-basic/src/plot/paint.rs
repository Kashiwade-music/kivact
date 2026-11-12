//! Role: Paints a plot: grid, axes, labels, series, legend and readout.

use super::decimate::runs;
use super::heatmap::{Cells, paint as paint_heatmap};
use super::input::{Readout, readout};
use super::model::{LineDash, PlotData, PlotModel, PlotSeries, PlotSeriesKind, PlotView};
use super::scale::{Map, maps, plot_area, within};
use super::ticks::{LogTick, axis_ticks, format_value, tick_spacing};
use crate::tokens::Tokens;
use kivact::draw::{Draw, Layer, LineCap, Stroke, TextBaseline, TextFormat};
use kivact::geometry::{Point, Rect, Size};
use kivact::i18n::Locale;
use kivact::style::{Color, TextAlign};

/// The distance between two x tick labels, in logical px.
const X_TICK_PX: f64 = 80.0;
/// The distance between two y tick labels, in logical px.
const Y_TICK_PX: f64 = 50.0;
/// The size of the axis texts, in logical px.
const TEXT: f64 = 12.0;
/// The longest share of the plot's width that a series name may take in the legend.
const LEGEND_NAME_SHARE: f64 = 0.4;
/// The gap between the legend box and the plot's right edge.
const LEGEND_MARGIN: f64 = 8.0;
/// The width of a legend color swatch, which the name follows.
const LEGEND_SWATCH: f64 = 16.0;
/// The mark that ends a shortened name.
const ELLIPSIS: &str = "\u{2026}";

/// Paint `model` at `view` over a canvas of `size` (logical px) in the theme
/// of `draw`, with no pointer. Its colors are the kit's tokens, its numbers
/// are written in `locale`, and it is the same picture a plot shows, so it
/// can go to an [`Offscreen`](kivact::draw::Offscreen) export.
///
/// The plot is drawn in the rectangle the canvas leaves for it: the
/// background, grid lines, axes, tick labels, axis titles, the title, the
/// series and the legend in the top right corner.
pub fn draw_plot(draw: &mut Draw, size: Size, model: &PlotModel, view: PlotView, locale: &Locale) {
    paint(draw, size, model, view, None, locale);
}

/// What one paint reads: the model and its view, the plot area with its
/// maps and ticks, and the tokens, colormap, size and locale of the frame.
struct Frame<'a> {
    model: &'a PlotModel,
    view: PlotView,
    size: Size,
    area: Rect,
    x: Map,
    y: Map,
    x_ticks: Vec<LogTick>,
    y_ticks: Vec<LogTick>,
    tokens: &'a Tokens,
    stops: &'a [[f64; 4]],
    locale: &'a Locale,
}

/// Paint the plot, with a crosshair and a readout when `hover` is inside it.
pub(super) fn paint(
    draw: &mut Draw,
    size: Size,
    model: &PlotModel,
    view: PlotView,
    hover: Option<Point>,
    locale: &Locale,
) {
    let area = plot_area(size, model);
    if area.width <= 0.0 || area.height <= 0.0 {
        return;
    }
    let tokens = draw.theme().ext::<Tokens>();
    let stops = colormap_stops(draw, &tokens);
    let (x, y) = maps(model, view, area);
    let frame = Frame {
        model,
        view,
        size,
        area,
        x,
        y,
        x_ticks: axis_ticks(
            model.x.scale,
            view.x_min,
            view.x_max,
            area.width / X_TICK_PX,
        ),
        y_ticks: axis_ticks(
            model.y.scale,
            view.y_min,
            view.y_max,
            area.height / Y_TICK_PX,
        ),
        tokens: &tokens,
        stops: &stops,
        locale,
    };
    draw.fill_rect(area, tokens.card);
    paint_grid(draw, &frame);
    paint_axes(draw, &frame);
    paint_title(draw, &frame);
    draw.layer(Layer::new().clip_rect(area), |draw| {
        paint_series_all(draw, &frame);
    });
    if model.legend && !model.series.is_empty() {
        paint_legend(draw, &frame);
    }
    if let Some(at) = hover.filter(|at| within(area, *at)) {
        paint_hover(draw, &frame, at);
    }
}

/// The heatmap colormap as RGBA stops in `0..=1`, from the lowest value to the highest.
fn colormap_stops(draw: &Draw, tokens: &Tokens) -> Vec<[f64; 4]> {
    tokens
        .heatmap
        .iter()
        .map(|color| {
            let c = color.resolve(draw.theme());
            [c.r, c.g, c.b, c.a]
        })
        .collect()
}

/// The color of the series at `index`: its own color, else the kit's chart color of the index.
fn series_color(series: &PlotSeries, index: usize, tokens: &Tokens) -> Color {
    series
        .color
        .unwrap_or(tokens.chart[index % tokens.chart.len()])
}

/// The text style of an axis or legend text.
fn text_style(size: f64, color: Color, align: TextAlign, baseline: TextBaseline) -> TextFormat {
    TextFormat {
        size,
        color,
        align,
        baseline,
        ..TextFormat::default()
    }
}

/// The grid lines of the major and the minor ticks.
fn paint_grid(draw: &mut Draw, f: &Frame) {
    for tick in &f.x_ticks {
        if let Some(x) = f.x.px(tick.value) {
            let at = Point::new(x, f.area.y);
            draw.line(
                at,
                Point::new(x, f.area.bottom()),
                grid_width(tick),
                f.tokens.border,
            );
        }
    }
    for tick in &f.y_ticks {
        if let Some(y) = f.y.px(tick.value) {
            let at = Point::new(f.area.x, y);
            draw.line(
                at,
                Point::new(f.area.right(), y),
                grid_width(tick),
                f.tokens.border,
            );
        }
    }
}

/// The width of a grid line: a major tick's is 1 px, a minor tick's half.
fn grid_width(tick: &LogTick) -> f64 {
    if tick.major { 1.0 } else { 0.5 }
}

/// The two axis lines, and the tick labels and titles beside them.
fn paint_axes(draw: &mut Draw, f: &Frame) {
    let color = f.tokens.muted_foreground;
    let bottom_left = Point::new(f.area.x, f.area.bottom());
    draw.line(
        bottom_left,
        Point::new(f.area.right(), f.area.bottom()),
        Stroke::new(1.0),
        color,
    );
    draw.line(
        bottom_left,
        Point::new(f.area.x, f.area.y),
        Stroke::new(1.0),
        color,
    );
    paint_tick_labels(draw, f);
    paint_axis_titles(draw, f);
}

/// The labels of the major ticks: under the x axis, and left of the y axis.
fn paint_tick_labels(draw: &mut Draw, f: &Frame) {
    let color = f.tokens.muted_foreground;
    let x_step = tick_spacing(&f.x_ticks, f.model.x.scale);
    let below = text_style(TEXT, color, TextAlign::Center, TextBaseline::Top);
    for tick in f.x_ticks.iter().filter(|t| t.major) {
        if let Some(x) = f.x.px(tick.value) {
            let text = format_value(tick.value, &f.model.x.format, x_step, f.locale);
            draw.text(&text, Point::new(x, f.area.bottom() + 6.0), below.clone());
        }
    }
    let y_step = tick_spacing(&f.y_ticks, f.model.y.scale);
    let beside = text_style(TEXT, color, TextAlign::End, TextBaseline::Middle);
    for tick in f.y_ticks.iter().filter(|t| t.major) {
        if let Some(y) = f.y.px(tick.value) {
            let text = format_value(tick.value, &f.model.y.format, y_step, f.locale);
            draw.text(&text, Point::new(f.area.x - 6.0, y), beside.clone());
        }
    }
}

/// The axis titles: the x title centered under the plot, the y title at its top left.
fn paint_axis_titles(draw: &mut Draw, f: &Frame) {
    let color = f.tokens.muted_foreground;
    if !f.model.x.label.is_empty() {
        let at = Point::new(f.area.x + f.area.width / 2.0, f.area.bottom() + 22.0);
        let style = text_style(TEXT, color, TextAlign::Center, TextBaseline::Top);
        draw.text(&f.model.x.label, at, style);
    }
    if !f.model.y.label.is_empty() {
        let at = Point::new(f.area.x + 4.0, f.area.y + 4.0);
        let style = text_style(TEXT, color, TextAlign::Start, TextBaseline::Top);
        draw.text(&f.model.y.label, at, style);
    }
}

/// The title, centered at the top of the canvas.
fn paint_title(draw: &mut Draw, f: &Frame) {
    if f.model.title.is_empty() {
        return;
    }
    let style = text_style(
        TEXT + 2.0,
        f.tokens.muted_foreground,
        TextAlign::Center,
        TextBaseline::Top,
    );
    draw.text(&f.model.title, Point::new(f.size.width / 2.0, 4.0), style);
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

/// Paint every series in order, each in its color, inside the plot area.
fn paint_series_all(draw: &mut Draw, f: &Frame) {
    for (index, series) in f.model.series.iter().enumerate() {
        let color = series_color(series, index, f.tokens);
        paint_series(draw, f, series, color);
    }
}

/// Paint one series of its kind.
fn paint_series(draw: &mut Draw, f: &Frame, series: &PlotSeries, color: Color) {
    match &series.kind {
        PlotSeriesKind::Line { x, y } => {
            let stroke = stroke_of(series);
            for run in runs(&pixels(f, x, y), f.area) {
                draw.polyline(run, &stroke, color);
            }
        }
        PlotSeriesKind::Points { x, y, radius } => {
            for at in pixels(f, x, y).into_iter().flatten() {
                draw.circle(at, *radius, color);
            }
        }
        PlotSeriesKind::Stems { x, y } => paint_stems(draw, f, series, x, y, color),
        PlotSeriesKind::Steps { edges, values } => paint_steps(draw, f, edges, values, color),
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
            paint_heatmap(draw, f.area, &f.x, &f.y, &cells, f.stops);
        }
    }
}

/// The plot pixels of the points `(x[i], y[i])`; a point the axes cannot show is `None`.
fn pixels(f: &Frame, x: &PlotData, y: &PlotData) -> Vec<Option<Point>> {
    x.as_slice()
        .iter()
        .zip(y.as_slice())
        .map(|(xv, yv)| {
            f.x.px(*xv)
                .zip(f.y.px(*yv))
                .map(|(px, py)| Point::new(px, py))
        })
        .collect()
}

/// The base line of the bars and stems: the pixel of the axis's lowest value,
/// or the bottom of the plot when it cannot be shown.
fn base_y(f: &Frame) -> f64 {
    f.y.px(f.view.y_min).unwrap_or(f.area.bottom())
}

/// A vertical line from the base to each point, with a dot at its top.
fn paint_stems(
    draw: &mut Draw,
    f: &Frame,
    series: &PlotSeries,
    x: &PlotData,
    y: &PlotData,
    color: Color,
) {
    let base = base_y(f);
    let stroke = stroke_of(series);
    for at in pixels(f, x, y).into_iter().flatten() {
        draw.line(Point::new(at.x, base), at, stroke.clone(), color);
        draw.circle(at, 3.0, color);
    }
}

/// The bars of a step series: each value is a bar between the edges around it,
/// standing on the base line.
fn paint_steps(draw: &mut Draw, f: &Frame, edges: &PlotData, values: &PlotData, color: Color) {
    let base = base_y(f);
    let edges = edges.as_slice();
    for (i, height) in values.as_slice().iter().enumerate() {
        let (Some(a), Some(b), Some(top)) = (
            edges.get(i).and_then(|e| f.x.px(*e)),
            edges.get(i + 1).and_then(|e| f.x.px(*e)),
            f.y.px(*height),
        ) else {
            continue;
        };
        let rect = Rect::new(a.min(b), top.min(base), (a - b).abs(), (base - top).abs());
        draw.fill_rect(rect, color);
    }
}

/// The legend in the top right corner of the plot. Its box is as wide as the
/// names it holds, and it stays inside the plot: a name wider than
/// [`LEGEND_NAME_SHARE`] of the plot is shortened.
fn paint_legend(draw: &mut Draw, f: &Frame) {
    let style = text_style(
        TEXT,
        f.tokens.foreground,
        TextAlign::Start,
        TextBaseline::Top,
    );
    let limit = f.area.width * LEGEND_NAME_SHARE;
    let names: Vec<(String, f64)> = f
        .model
        .series
        .iter()
        .map(|series| {
            let name = fit_name(&series.name, limit, |text| {
                draw.measure_text(text, &style).size.width
            });
            let width = draw.measure_text(&name, &style).size.width;
            (name, width)
        })
        .collect();
    let widest = names.iter().map(|(_, width)| *width).fold(0.0, f64::max);
    let left = (f.area.right() - LEGEND_MARGIN - LEGEND_SWATCH - widest).max(f.area.x);
    for (index, (series, (name, _))) in f.model.series.iter().zip(&names).enumerate() {
        let y = f.area.y + 8.0 + index as f64 * 16.0;
        let swatch = Rect::new(left, y + 2.0, 10.0, 10.0);
        draw.fill_rect(swatch, series_color(series, index, f.tokens));
        draw.text(name, Point::new(left + LEGEND_SWATCH, y), style.clone());
    }
}

/// The name, cut to its longest prefix that ends with an ellipsis and measures at most `limit`.
/// A name that already fits is returned as it is. Only the ellipsis is left when even it does not
/// fit.
fn fit_name(name: &str, limit: f64, measure: impl Fn(&str) -> f64) -> String {
    if measure(name) <= limit {
        return name.to_string();
    }
    let chars: Vec<char> = name.chars().collect();
    (0..chars.len())
        .rev()
        .map(|kept| chars[..kept].iter().collect::<String>() + ELLIPSIS)
        .find(|candidate| measure(candidate) <= limit)
        .unwrap_or_else(|| ELLIPSIS.to_string())
}

/// The crosshair through the pointer, a dot on the nearest point and the readout card.
fn paint_hover(draw: &mut Draw, f: &Frame, at: Point) {
    let ring = f.tokens.ring;
    draw.line(
        Point::new(at.x, f.area.y),
        Point::new(at.x, f.area.bottom()),
        Stroke::new(1.0),
        ring,
    );
    draw.line(
        Point::new(f.area.x, at.y),
        Point::new(f.area.right(), at.y),
        Stroke::new(1.0),
        ring,
    );
    let Some(read) = readout(f.model, f.view, f.area, at) else {
        return;
    };
    draw.circle(read.at, 4.0, ring);
    paint_card(draw, f, &read, at);
}

/// The lines of the readout card: the series name, then the values.
fn readout_lines(f: &Frame, read: &Readout) -> Vec<String> {
    let mut lines = vec![
        read.name.clone(),
        format!(
            "x: {}",
            format_value(read.x, &f.model.x.format, 0.0, f.locale)
        ),
        format!(
            "y: {}",
            format_value(read.y, &f.model.y.format, 0.0, f.locale)
        ),
    ];
    if let Some(z) = read.z {
        lines.push(format!(
            "value: {}",
            format_value(z, &f.model.y.format, 0.0, f.locale)
        ));
    }
    lines
}

/// The readout card beside the pointer, turned to the left at the right edge
/// and kept inside the plot.
fn paint_card(draw: &mut Draw, f: &Frame, read: &Readout, at: Point) {
    let lines = readout_lines(f, read);
    let style = text_style(
        TEXT,
        f.tokens.popover_foreground,
        TextAlign::Start,
        TextBaseline::Top,
    );
    let width = lines
        .iter()
        .map(|text| draw.measure_text(text, &style).size.width)
        .fold(0.0, f64::max);
    let height = 16.0 * lines.len() as f64 + 8.0;
    let card_width = width + 16.0;
    let mut x = at.x + 12.0;
    if x + card_width > f.area.right() {
        x = at.x - 12.0 - card_width;
    }
    let y = (at.y + 12.0).min(f.area.bottom() - height).max(f.area.y);
    let card = Rect::new(x, y, card_width, height);
    draw.fill_rect(card, f.tokens.popover);
    draw.stroke_rect(card, Stroke::new(1.0), f.tokens.border);
    for (i, text) in lines.iter().enumerate() {
        let at = Point::new(x + 8.0, y + 4.0 + 16.0 * i as f64);
        draw.text(text, at, style.clone());
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the legend's name fitting.

    use super::*;

    /// Six px for each character, so the widths in the assertions are easy to read.
    fn six_px(text: &str) -> f64 {
        text.chars().count() as f64 * 6.0
    }

    #[test]
    fn legend_names_are_cut_to_the_limit_with_an_ellipsis() {
        assert_eq!(fit_name("Gain", 24.0, six_px), "Gain");
        assert_eq!(
            fit_name("Very long series name", 36.0, six_px),
            "Very \u{2026}"
        );
        assert_eq!(fit_name("abc", 3.0, six_px), ELLIPSIS);
        assert_eq!(fit_name("", 0.0, six_px), "");
    }
}
