//! Role: Paint and hit-test the radar chart.

use super::model::{Frame, Hit, HitRow, series_color};
use super::polar::{MARGIN, locate, polar};
use super::scale::{Domain, text_width};
use crate::tokens::Tokens;
use kivact::draw::{Draw, LineJoin, Path, Point, Size, Stroke, TextBaseline, TextStyle};
use kivact::style::TextAlign;

/// The gap between the outer ring and the category labels.
const LABEL_GAP: f64 = 10.0;
/// The height the labels above and below the web need.
const LABEL_ROOM: f64 = 28.0;

/// The web: its middle, radius, value scale and the angle of each axis.
struct Web {
    center: Point,
    radius: f64,
    domain: Domain,
    axes: usize,
}

impl Web {
    fn new(size: Size, f: &Frame) -> Self {
        let widest = f
            .rows
            .iter()
            .map(|row| text_width(&row.label, 12.0))
            .fold(0.0, f64::max);
        let radius = if f.axis {
            (size.height / 2.0 - LABEL_ROOM).min(size.width / 2.0 - widest - LABEL_GAP - MARGIN)
        } else {
            size.width.min(size.height) / 2.0 - MARGIN
        };
        let highest = (0..f.config.series.len())
            .flat_map(|series| (0..f.rows.len()).map(move |row| (row, series)))
            .map(|(row, series)| f.value(row, series))
            .fold(0.0, f64::max);
        Self {
            center: Point::new(size.width / 2.0, size.height / 2.0),
            radius: radius.max(16.0),
            domain: Domain::nice(0.0, highest),
            axes: f.rows.len().max(1),
        }
    }

    /// The angle of axis `index`: the first axis points up.
    fn angle(&self, index: usize) -> f64 {
        -90.0 + 360.0 * index as f64 / self.axes as f64
    }

    /// The point of axis `index` at `value`.
    fn point(&self, index: usize, value: f64) -> Point {
        let fraction = self.domain.fraction(value).clamp(0.0, 1.0);
        polar(self.center, self.radius * fraction, self.angle(index))
    }
}

/// Paint the chart; `hit` is the axis under the pointer.
pub(super) fn paint(d: &mut Draw, size: Size, f: &Frame, hit: Option<&Hit>) {
    let tokens = d.theme().get::<Tokens>();
    let web = Web::new(size, f);
    if f.grid {
        paint_web(d, &web, &tokens);
    }
    if f.axis {
        paint_labels(d, &web, f, &tokens);
    }
    if let Some(hit) = hit {
        let spoke = Path::new().move_to(web.center).line_to(polar(
            web.center,
            web.radius,
            web.angle(hit.index),
        ));
        d.stroke_path(&spoke, &Stroke::new(1.0), tokens.ring);
    }
    let stroke = Stroke::new(2.0).join(LineJoin::Round);
    for series in 0..f.config.series.len() {
        let color = series_color(f.config, series, &tokens);
        let points: Vec<Point> = (0..f.rows.len())
            .map(|row| web.point(row, f.value(row, series)))
            .collect();
        let Some(first) = points.first() else {
            continue;
        };
        let mut shape = Path::new().move_to(*first);
        for point in &points[1..] {
            shape = shape.line_to(*point);
        }
        let shape = shape.close();
        d.fill_path(&shape, color.with_alpha(0.6));
        d.stroke_path(&shape, &stroke, color);
        for (row, point) in points.iter().enumerate() {
            let hovered = hit.is_some_and(|hit| hit.index == row);
            d.circle(*point, if hovered { 5.0 } else { 3.0 }, color);
        }
    }
}

fn paint_web(d: &mut Draw, web: &Web, tokens: &Tokens) {
    let stroke = Stroke::new(1.0);
    for tick in web.domain.ticks() {
        if tick <= web.domain.lo {
            continue;
        }
        let mut ring = Path::new().move_to(web.point(0, tick));
        for axis in 1..web.axes {
            ring = ring.line_to(web.point(axis, tick));
        }
        d.stroke_path(&ring.close(), &stroke, tokens.border);
    }
    for axis in 0..web.axes {
        let spoke = Path::new()
            .move_to(web.center)
            .line_to(web.point(axis, web.domain.hi));
        d.stroke_path(&spoke, &stroke, tokens.border);
    }
}

fn paint_labels(d: &mut Draw, web: &Web, f: &Frame, tokens: &Tokens) {
    for (axis, row) in f.rows.iter().enumerate() {
        let degrees = web.angle(axis);
        let at = polar(web.center, web.radius + LABEL_GAP, degrees);
        let (cos, sin) = (degrees.to_radians().cos(), degrees.to_radians().sin());
        let align = if cos > 0.3 {
            TextAlign::Start
        } else if cos < -0.3 {
            TextAlign::End
        } else {
            TextAlign::Center
        };
        let baseline = if sin < -0.7 {
            TextBaseline::Bottom
        } else if sin > 0.7 {
            TextBaseline::Top
        } else {
            TextBaseline::Middle
        };
        let style = TextStyle {
            size: 12.0,
            color: tokens.muted_foreground,
            align,
            baseline,
            ..TextStyle::default()
        };
        d.text(&row.label, at, style);
    }
}

/// The axis nearest to `at` while the pointer is over the web.
pub(super) fn hit(size: Size, f: &Frame, at: Point) -> Option<Hit> {
    let web = Web::new(size, f);
    if f.rows.is_empty() {
        return None;
    }
    let (distance, angle) = locate(web.center, at);
    if distance > web.radius + LABEL_GAP {
        return None;
    }
    let step = 360.0 / web.axes as f64;
    let index = ((angle + step / 2.0) / step) as usize % web.axes;
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
