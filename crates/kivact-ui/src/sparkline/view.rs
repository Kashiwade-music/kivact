//! Role: The sparkline component: a small line chart of values.

use crate::layer::with_base_recipe_style;
use crate::sparkline::look::{SparklineState, SparklineTheme, Summary};
use kivact_core::custom::Caller;
use kivact_core::i18n::{FractionDigits, Grouping, Locale, NumberStyle, message_with, use_locale};
use kivact_core::state::{Value, memo};
use kivact_core::view::{Role, node};
use kivact_macros::component;
use std::rc::Rc;

/// A small line chart of `values`, drawn in the theme's sparkline look.
///
/// Values are spaced evenly across the width, the highest at the top and the
/// lowest at the bottom of the box. A value that is not finite breaks the line
/// there. With fewer than two finite values the chart is hidden, so an empty
/// chart slot collapses. A flat series is drawn as a horizontal line in the
/// middle. `label` names the chart for assistive technology: the name is the
/// label, and the description gives the count of values, their range and the
/// last value (`sparkline.description`, formatted in the app's locale).
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
#[component]
pub fn sparkline(
    values: Value<Vec<f64>>,
    label: Option<Value<String>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    // The one memo: the summary. The state copies the values once per change.
    let summary = memo(move || values.with(|values| Summary::of(values)));
    let state = memo(move || SparklineState {
        summary: summary.get(),
        values: values.with(|values| Rc::from(values.as_slice())),
    });
    let root = with_base_recipe_style(
        node().hidden(move || summary.get().count < 2),
        |r: &SparklineTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    );
    match label {
        Some(label) => root
            .role(Role::Image)
            .name(label)
            .description(description(summary)),
        None => root,
    }
}

/// The reactive description of a labelled chart, in the app's locale.
fn description(summary: Value<Summary>) -> Value<String> {
    let locale = use_locale();
    message_with(
        "sparkline.description",
        "{count} values from {min} to {max}, last {last}",
        move || {
            let summary = summary.get();
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
