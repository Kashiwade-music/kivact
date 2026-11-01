//! Role: Paint and hit-test the bar, stacked bar, line and area charts.

use super::model::{ChartKind, Frame, Hit, HitRow, series_color};
use super::scale::{Domain, text_width};
use crate::tokens::Tokens;
use kivact::draw::{
    Draw, LineCap, LineJoin, Path, Point, Rect, Size, Stroke, TextBaseline, TextStyle,
};
use kivact::style::TextAlign;

/// The empty edge around the plot.
const MARGIN: f32 = 8.0;
/// The height of the category labels under the plot, with their gap.
const AXIS_HEIGHT: f32 = 24.0;
/// The gap between the value labels and the plot.
const LABEL_GAP: f32 = 8.0;
/// The gap between the bars of one category.
const BAR_GAP: f32 = 4.0;
/// The share of a category band the bars of one category use.
const BAR_SHARE: f32 = 0.8;

/// Where the plot lies and how values and categories map onto it.
struct Layout {
    plot: Rect,
    domain: Domain,
    bands: usize,
    horizontal: bool,
}

impl Layout {
    fn new(size: Size, f: &Frame) -> Self {
        let series = f.config.series.len();
        let stacked = f.kind == ChartKind::StackedBar;
        let horizontal = f.horizontal && matches!(f.kind, ChartKind::Bar | ChartKind::StackedBar);
        let (mut lo, mut hi) = (0.0f64, 0.0f64);
        for row in 0..f.rows.len() {
            let (mut up, mut down) = (0.0f64, 0.0f64);
            for s in 0..series {
                let value = f.value(row, s);
                if stacked {
                    if value >= 0.0 {
                        up += value;
                    } else {
                        down += value;
                    }
                } else {
                    up = up.max(value);
                    down = down.min(value);
                }
            }
            hi = hi.max(up);
            lo = lo.min(down);
        }
        let domain = Domain::nice(lo, hi);
        let gutter = if !f.axis {
            0.0
        } else if horizontal {
            let widest = f
                .rows
                .iter()
                .map(|row| text_width(&row.label, 12.0))
                .fold(0.0, f32::max);
            widest.min(size.width * 0.4) + LABEL_GAP
        } else {
            let widest = domain
                .ticks()
                .iter()
                .map(|tick| text_width(&f.format(*tick), 12.0))
                .fold(0.0, f32::max);
            widest + LABEL_GAP
        };
        let left = MARGIN + gutter;
        let right = if f.axis && horizontal {
            MARGIN + 12.0
        } else {
            MARGIN
        };
        let bottom = MARGIN + if f.axis { AXIS_HEIGHT } else { 0.0 };
        Self {
            plot: Rect::new(
                left,
                MARGIN,
                (size.width - left - right).max(1.0),
                (size.height - MARGIN - bottom).max(1.0),
            ),
            domain,
            bands: f.rows.len().max(1),
            horizontal,
        }
    }

    /// The size of one category along its axis.
    fn band(&self) -> f32 {
        let length = if self.horizontal {
            self.plot.height
        } else {
            self.plot.width
        };
        length / self.bands as f32
    }

    /// The middle of category `index` along its axis.
    fn center(&self, index: usize) -> f32 {
        let origin = if self.horizontal {
            self.plot.y
        } else {
            self.plot.x
        };
        origin + (index as f32 + 0.5) * self.band()
    }

    /// Where `value` lies along the value axis.
    fn position(&self, value: f64) -> f32 {
        let fraction = self.domain.fraction(value);
        if self.horizontal {
            self.plot.x + fraction * self.plot.width
        } else {
            self.plot.y + self.plot.height - fraction * self.plot.height
        }
    }

    /// The point of category `index` at `value`.
    fn point(&self, index: usize, value: f64) -> Point {
        if self.horizontal {
            Point::new(self.position(value), self.center(index))
        } else {
            Point::new(self.center(index), self.position(value))
        }
    }

    /// A bar `across` thick centered on `center` and spanning `from` to `to` in value.
    fn bar(&self, center: f32, across: f32, from: f64, to: f64) -> Rect {
        let (a, b) = (self.position(from), self.position(to));
        let (near, length) = (a.min(b), (a - b).abs());
        if self.horizontal {
            Rect::new(near, center - across / 2.0, length, across)
        } else {
            Rect::new(center - across / 2.0, near, across, length)
        }
    }
}

/// `v` moved to the middle of a device pixel so a 1 px line stays crisp.
fn crisp(v: f32) -> f32 {
    v.floor() + 0.5
}

fn label_style(tokens: &Tokens, align: TextAlign, baseline: TextBaseline) -> TextStyle {
    TextStyle {
        size: 12.0,
        color: tokens.muted_foreground,
        align,
        baseline,
        ..TextStyle::default()
    }
}

/// Paint the chart; `hit` is the row under the pointer.
pub(super) fn paint(d: &mut Draw, size: Size, f: &Frame, hit: Option<&Hit>) {
    let tokens = d.theme().get::<Tokens>();
    let layout = Layout::new(size, f);
    paint_grid(d, &layout, f, &tokens);
    if f.axis {
        paint_labels(d, &layout, f, &tokens);
    }
    if let Some(hit) = hit {
        paint_cursor(d, &layout, f, &tokens, hit.index);
    }
    match f.kind {
        ChartKind::Bar => paint_bars(d, &layout, f, &tokens),
        ChartKind::StackedBar => paint_stacks(d, &layout, f, &tokens),
        _ => paint_lines(d, &layout, f, &tokens, hit.map(|hit| hit.index)),
    }
}

fn paint_grid(d: &mut Draw, l: &Layout, f: &Frame, tokens: &Tokens) {
    if !f.grid {
        return;
    }
    let color = tokens.border.with_alpha(0.5);
    for tick in l.domain.ticks() {
        let at = l.position(tick);
        let (from, to) = if l.horizontal {
            let x = crisp(at);
            (
                Point::new(x, l.plot.y),
                Point::new(x, l.plot.y + l.plot.height),
            )
        } else {
            let y = crisp(at);
            (
                Point::new(l.plot.x, y),
                Point::new(l.plot.x + l.plot.width, y),
            )
        };
        d.line(from, to, Stroke::new(1.0), color);
    }
}

fn paint_labels(d: &mut Draw, l: &Layout, f: &Frame, tokens: &Tokens) {
    let below = l.plot.y + l.plot.height + 6.0;
    for tick in l.domain.ticks() {
        let at = l.position(tick);
        let text = f.format(tick);
        if l.horizontal {
            let style = label_style(tokens, TextAlign::Center, TextBaseline::Top);
            d.text(&text, Point::new(at, below), style);
        } else {
            let style = label_style(tokens, TextAlign::End, TextBaseline::Middle);
            d.text(&text, Point::new(l.plot.x - LABEL_GAP, at), style);
        }
    }
    // Thin the category labels out until they no longer collide.
    let widest = f
        .rows
        .iter()
        .map(|row| text_width(&row.label, 12.0))
        .fold(0.0, f32::max);
    let needed = if l.horizontal { 20.0 } else { widest + 12.0 };
    let stride = (needed / l.band()).ceil().max(1.0) as usize;
    for (i, row) in f.rows.iter().enumerate().step_by(stride) {
        if l.horizontal {
            let style = label_style(tokens, TextAlign::End, TextBaseline::Middle);
            d.text(
                &row.label,
                Point::new(l.plot.x - LABEL_GAP, l.center(i)),
                style,
            );
        } else {
            let style = label_style(tokens, TextAlign::Center, TextBaseline::Top);
            d.text(&row.label, Point::new(l.center(i), below), style);
        }
    }
}

/// The hovered category: a muted band behind bars, a rule behind lines.
fn paint_cursor(d: &mut Draw, l: &Layout, f: &Frame, tokens: &Tokens, index: usize) {
    if index >= f.rows.len() {
        return;
    }
    let center = l.center(index);
    if matches!(f.kind, ChartKind::Bar | ChartKind::StackedBar) {
        let band = l.band();
        let rect = if l.horizontal {
            Rect::new(l.plot.x, center - band / 2.0, l.plot.width, band)
        } else {
            Rect::new(center - band / 2.0, l.plot.y, band, l.plot.height)
        };
        d.fill_rect(rect, tokens.muted);
    } else {
        let x = crisp(center);
        d.line(
            Point::new(x, l.plot.y),
            Point::new(x, l.plot.y + l.plot.height),
            Stroke::new(1.0),
            tokens.border,
        );
    }
}

fn paint_bars(d: &mut Draw, l: &Layout, f: &Frame, tokens: &Tokens) {
    let series = f.config.series.len();
    if series == 0 {
        return;
    }
    let count = series as f32;
    let group = l.band() * BAR_SHARE;
    let across = ((group - BAR_GAP * (count - 1.0)) / count).max(1.0);
    let width = across * count + BAR_GAP * (count - 1.0);
    for row in 0..f.rows.len() {
        let first = l.center(row) - width / 2.0 + across / 2.0;
        for s in 0..series {
            let value = f.value(row, s);
            if value == 0.0 {
                continue;
            }
            let center = first + s as f32 * (across + BAR_GAP);
            let bar = l.bar(center, across, 0.0, value);
            d.fill_rect(bar, series_color(f.config, s, tokens));
        }
    }
}

fn paint_stacks(d: &mut Draw, l: &Layout, f: &Frame, tokens: &Tokens) {
    let across = (l.band() * BAR_SHARE).max(1.0);
    for row in 0..f.rows.len() {
        let (mut up, mut down) = (0.0f64, 0.0f64);
        for s in 0..f.config.series.len() {
            let value = f.value(row, s);
            if value == 0.0 {
                continue;
            }
            let base = if value >= 0.0 { &mut up } else { &mut down };
            let from = *base;
            *base += value;
            let bar = l.bar(l.center(row), across, from, *base);
            d.fill_rect(bar, series_color(f.config, s, tokens));
        }
    }
}

fn paint_lines(d: &mut Draw, l: &Layout, f: &Frame, tokens: &Tokens, hovered: Option<usize>) {
    let stroke = Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round);
    let area = f.kind == ChartKind::Area;
    for s in 0..f.config.series.len() {
        let color = series_color(f.config, s, tokens);
        let points: Vec<Point> = (0..f.rows.len())
            .map(|row| l.point(row, f.value(row, s)))
            .collect();
        let (Some(first), Some(last)) = (points.first(), points.last()) else {
            continue;
        };
        if area {
            let base = l.position(0.0);
            let mut fill = Path::new()
                .move_to(Point::new(first.x, base))
                .line_to(*first);
            for point in &points[1..] {
                fill = fill.line_to(*point);
            }
            let fill = fill.line_to(Point::new(last.x, base)).close();
            d.fill_path(&fill, color.with_alpha(0.4));
        }
        let mut line = Path::new().move_to(*first);
        for point in &points[1..] {
            line = line.line_to(*point);
        }
        d.stroke_path(&line, &stroke, color);
        if !area || points.len() == 1 {
            for point in &points {
                d.circle(*point, 3.0, color);
            }
        }
        if let Some(point) = hovered.and_then(|index| points.get(index)) {
            d.circle(*point, 6.0, tokens.background);
            d.circle(*point, 4.0, color);
        }
    }
}

/// The category under `at`, with every series' value.
pub(super) fn hit(size: Size, f: &Frame, at: Point) -> Option<Hit> {
    let l = Layout::new(size, f);
    let inside = at.x >= l.plot.x
        && at.x <= l.plot.x + l.plot.width
        && at.y >= l.plot.y
        && at.y <= l.plot.y + l.plot.height;
    if f.rows.is_empty() || !inside {
        return None;
    }
    let along = if l.horizontal {
        at.y - l.plot.y
    } else {
        at.x - l.plot.x
    };
    let index = ((along / l.band()) as usize).min(f.rows.len() - 1);
    Some(Hit {
        index,
        title: Some(f.rows[index].label.clone()),
        rows: f
            .config
            .series
            .iter()
            .enumerate()
            .map(|(series, config)| HitRow {
                series,
                label: config.label.clone(),
                value: f.value(index, series),
            })
            .collect(),
    })
}
