//! Role: The sparkline recipe state, recipe table, line paint and default looks.

use crate::recipe::Look;
use crate::recipe::recipe_table;
use kivact_core::draw::{Draw, LineCap, LineJoin, Path, Stroke};
use kivact_core::geometry::{Point, Size};
use kivact_core::style::{Color, Length, ThemeVar, color, linear_gradient};
use kivact_core::style::{Style, Theme, pct, style};
use std::rc::Rc;

/// The color of a sparkline's line.
pub static LINE_VAR: ThemeVar<Color> = ThemeVar::alias("sparkline.line", &color::ACCENT_VAR);
/// [`LINE_VAR`] as a color for a style.
pub const LINE: Color = Color::var(&LINE_VAR);
/// The width of a sparkline's line, in logical px.
pub static LINE_WIDTH_VAR: ThemeVar<f64> = ThemeVar::new("sparkline.line_width", 1.5);
/// [`LINE_WIDTH_VAR`] as a length for a style.
pub const LINE_WIDTH: Length = Length::var(&LINE_WIDTH_VAR);
/// The inset of a sparkline's points from its box's edges, in logical px.
pub static INSET_VAR: ThemeVar<f64> = ThemeVar::new("sparkline.inset", 0.0);
/// [`INSET_VAR`] as a length for a style.
pub const INSET: Length = Length::var(&INSET_VAR);

/// What fills the area under a sparkline's line.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SparklineFill {
    /// No fill: the line alone.
    #[default]
    None,
    /// A solid fill of this color.
    Solid(Color),
    /// A fill of this color that fades toward the baseline.
    Fade(Color),
}

/// The count, range and last of the finite values of a series.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Summary {
    pub(crate) count: usize,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) last: f64,
}

impl Summary {
    /// The summary of the finite values; a series without any has zeros.
    pub(crate) fn of(values: &[f64]) -> Self {
        let mut summary = Summary {
            count: 0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            last: 0.0,
        };
        for value in values.iter().copied().filter(|v| v.is_finite()) {
            summary.count += 1;
            summary.min = summary.min.min(value);
            summary.max = summary.max.max(value);
            summary.last = value;
        }
        if summary.count == 0 {
            summary.min = 0.0;
            summary.max = 0.0;
        }
        summary
    }
}

/// The state a sparkline's recipes read: the summary of its values and the
/// values, which the line is drawn from.
#[derive(Clone, Debug, PartialEq)]
pub struct SparklineState {
    pub(crate) summary: Summary,
    pub(crate) values: Rc<[f64]>,
}

/// The line of a sparkline in `color`, with the area of its settings: the
/// paint of the default recipe. Value `i` sits at `width * i / (n - 1)`; the
/// highest value is at the theme's inset from the top and the lowest at the
/// inset from the bottom; a flat series is drawn in the middle.
pub fn sparkline_line(
    look: &Look<SparklineState>,
    color: Color,
) -> impl Fn(&mut Draw, Size) + 'static {
    let (values, summary) = (look.values.clone(), look.summary);
    move |d: &mut Draw, size: Size| {
        paint(d, size.width, size.height, &values, summary, color);
    }
}

pub(crate) fn sparkline_root(_: &Theme, look: &Look<SparklineState>) -> Style {
    style()
        .width(pct(100))
        .height(32)
        .min_width(1)
        .paint(sparkline_line(look, LINE))
}

recipe_table! {
    /// The recipes of a [`sparkline`](super::sparkline).
    SparklineTheme {
        /// The chart's box: its size, and the line and its area drawn in it. The theme variables and settings choose their colors and widths.
        root(root_recipe): SparklineState = sparkline_root,
    }
    settings {
        /// Whether the line's ends are round; see the line's width in `sparkline.line_width`.
        round(round_value): bool = false,
        /// What fills the area under the line; see [`SparklineFill`].
        fill(fill_value): SparklineFill = SparklineFill::None,
    }
}

/// Draw the line and the area of each run of finite values in a box,
/// straight into the paths. A value that is not finite ends the run before it.
fn paint(d: &mut Draw, width: f64, height: f64, values: &[f64], summary: Summary, color: Color) {
    if summary.count < 2 {
        return;
    }
    let theme = d.theme().clone();
    let inset = theme.get(&INSET_VAR);
    let span = summary.max - summary.min;
    let last_index = (values.len().max(2) - 1) as f64;
    let point = move |i: usize| {
        let t = if span > 0.0 {
            (values[i] - summary.min) / span
        } else {
            0.5
        };
        Point::new(
            width * i as f64 / last_index,
            inset + (1.0 - t) * (height - 2.0 * inset),
        )
    };
    let mut start = 0;
    while start < values.len() {
        let end = values[start..]
            .iter()
            .position(|value| !value.is_finite())
            .map_or(values.len(), |at| start + at);
        if end - start >= 2 {
            draw_run(d, (start..end).map(point), height, &theme, color);
        }
        start = end + 1;
    }
}

/// Draw one run of at least two points: the area under it down to the
/// bottom edge, then its line.
fn draw_run(
    d: &mut Draw,
    points: impl Iterator<Item = Point> + Clone,
    height: f64,
    theme: &Theme,
    color: Color,
) {
    let mut ends = points.clone();
    let Some(first) = ends.next() else {
        return;
    };
    let last = ends.last().unwrap_or(first);
    let line = points
        .clone()
        .enumerate()
        .fold(Path::new(), |path, (i, p)| {
            if i == 0 {
                path.move_to(p)
            } else {
                path.line_to(p)
            }
        });
    let area = points
        .fold(
            Path::new().move_to(Point::new(first.x, height)),
            |path, p| path.line_to(p),
        )
        .line_to(Point::new(last.x, height))
        .close();
    fill(d, &area, *theme.ext::<SparklineTheme>().fill_value());
    let stroke = Stroke::new(theme.get(&LINE_WIDTH_VAR));
    let stroke = if *theme.ext::<SparklineTheme>().round_value() {
        stroke.cap(LineCap::Round).join(LineJoin::Round)
    } else {
        stroke
    };
    d.stroke_path(&line, &stroke, color);
}

/// Fill the area under a run with the theme's fill.
fn fill(d: &mut Draw, area: &Path, fill: SparklineFill) {
    match fill {
        SparklineFill::None => {}
        SparklineFill::Solid(color) => d.fill_path(area, color),
        SparklineFill::Fade(color) => d.fill_path(
            area,
            linear_gradient(180)
                .stop(color, pct(0))
                .stop(Color::TRANSPARENT, pct(100)),
        ),
    }
}
