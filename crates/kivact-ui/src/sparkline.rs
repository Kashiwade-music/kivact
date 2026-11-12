//! Role: Draw a standard sparkline, a small line chart of numbers, with no state.
//!
//! The chart is one node. Its box is the recipe's size and its line and area
//! are the theme's [`SparklineTokens`](crate::SparklineTokens); the values are
//! read where they are drawn, so a change of values redraws the line without
//! a state of the part. With a [`Sparkline::label`] it is an image named for
//! assistive technology with a description of its values; without one it is
//! decorative, which suits a statistic's chart slot.
use crate::UiTokens;
use crate::layer::with_base_recipe_style;
use crate::sparkline_tokens::SparklineFill;
use crate::variants::{Variant, Variants};
use kivact_core::draw::{Draw, LineCap, LineJoin, Path, Stroke, linear_gradient};
use kivact_core::geometry::Point;
use kivact_core::i18n::{FractionDigits, Grouping, Locale, NumberStyle, message_with, use_locale};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::style::{Color, pct};
use kivact_core::view::{Element, IntoView, Node, Role, View, node};

/// A small line chart of numbers, created by [`sparkline`].
pub struct Sparkline {
    root: Node,
    values: Value<Vec<f64>>,
    label: Option<Value<String>>,
    variants: Variants,
}

/// A small line chart of `values`, drawn in the theme's sparkline look.
///
/// Values are spaced evenly across the width, the highest at the top and the
/// lowest at the bottom of the box. A value that is not finite breaks the line
/// there. With fewer than two finite values the chart is hidden, so an empty
/// chart slot collapses. A flat series is drawn as a horizontal line in the
/// middle.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::sparkline;
///
/// fn trend() -> impl IntoView {
///     let samples = signal(vec![3.0, 4.5, 4.0, 6.0, 5.5]);
///     sparkline(move || samples.get()).label("Visitors per day")
/// }
/// ```
#[track_caller]
pub fn sparkline(values: impl IntoValue<Vec<f64>>) -> Sparkline {
    let values: Value<Vec<f64>> = values.into_value();
    Sparkline {
        root: node()
            .hidden(move || Summary::of(&values.get()).count < 2)
            .draw(move |d, s| paint(d, s.width, s.height, &values.get())),
        values,
        label: None,
        variants: Variants::default(),
    }
}

impl Sparkline {
    /// Name the chart for assistive technology. The name is the label, and the
    /// description gives the count of values, their range and the last value
    /// (`sparkline.description`, formatted in the app's locale). Without a
    /// label the chart is decorative; a statistic's `chart(..)` slot is one
    /// when the statistic already names it.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }

    /// Give the chart an appearance that its recipe reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same type
    /// replaces the earlier one; a theme that does not know the type draws the
    /// plain look.
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}

impl Element for Sparkline {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Sparkline {
    fn into_view(self) -> View {
        let Sparkline {
            mut root,
            values,
            label,
            variants,
        } = self;
        if let Some(label) = label {
            root = root
                .role(Role::Image)
                .name(label)
                .description(description(values));
        }
        with_base_recipe_style(
            root,
            |r| r.sparkline_recipes().root_recipe(),
            &variants,
            || (),
        )
        .into_view()
    }
}

/// The count, range and last of the finite values of a series.
struct Summary {
    count: usize,
    min: f64,
    max: f64,
    last: f64,
}

impl Summary {
    /// The summary of the finite values; a series without any has zeros.
    fn of(values: &[f64]) -> Self {
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

/// The reactive description of a labelled chart, in the app's locale.
fn description(values: Value<Vec<f64>>) -> Value<String> {
    let locale = use_locale();
    message_with(
        "sparkline.description",
        "{count} values from {min} to {max}, last {last}",
        move || {
            let summary = Summary::of(&values.get());
            locale.with(|locale| {
                [
                    ("count", number(locale, summary.count as f64)),
                    ("min", number(locale, summary.min)),
                    ("max", number(locale, summary.max)),
                    ("last", number(locale, summary.last)),
                ]
            })
        },
    )
}

/// A number in the locale, with up to three decimals.
fn number(locale: &Locale, value: f64) -> String {
    locale.format_number(
        value,
        NumberStyle {
            fraction_digits: FractionDigits::Range { min: 0, max: 3 },
            grouping: Grouping::Locale,
        },
    )
}

/// Draw the line and the area of each run of finite values in a box.
fn paint(d: &mut Draw, width: f64, height: f64, values: &[f64]) {
    let summary = Summary::of(values);
    if summary.count < 2 {
        return;
    }
    let tokens = d.theme().ext::<UiTokens>().sparkline.clone();
    for run in runs(values, &summary, (width, height), tokens.inset) {
        if run.len() < 2 {
            continue;
        }
        let line = run.iter().enumerate().fold(Path::new(), |path, (i, p)| {
            if i == 0 {
                path.move_to(*p)
            } else {
                path.line_to(*p)
            }
        });
        let (first, last) = (run[0], run[run.len() - 1]);
        let area = run
            .iter()
            .fold(
                Path::new().move_to(Point::new(first.x, height)),
                |path, p| path.line_to(*p),
            )
            .line_to(Point::new(last.x, height))
            .close();
        fill(d, &area, tokens.fill);
        let stroke = Stroke::new(tokens.line_width);
        let stroke = if tokens.round {
            stroke.cap(LineCap::Round).join(LineJoin::Round)
        } else {
            stroke
        };
        d.stroke_path(&line, &stroke, tokens.line);
    }
}

/// The points of each run of finite values. Value `i` sits at `width * i / (n - 1)`;
/// the highest value is at `inset` from the top and the lowest at `inset` from
/// the bottom. A flat series is drawn in the middle. A value that is not finite
/// ends the run before it.
fn runs(values: &[f64], summary: &Summary, size: (f64, f64), inset: f64) -> Vec<Vec<Point>> {
    let (width, height) = size;
    let span = summary.max - summary.min;
    let last_index = values.len().max(2) - 1;
    let mut runs = Vec::new();
    let mut run = Vec::new();
    for (i, value) in values.iter().enumerate() {
        if !value.is_finite() {
            runs.push(std::mem::take(&mut run));
            continue;
        }
        let t = if span > 0.0 {
            (value - summary.min) / span
        } else {
            0.5
        };
        let x = width * i as f64 / last_index as f64;
        let y = inset + (1.0 - t) * (height - 2.0 * inset);
        run.push(Point::new(x, y));
    }
    runs.push(run);
    runs
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
