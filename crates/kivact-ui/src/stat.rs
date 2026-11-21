//! Role: Present a named statistic with optional delta, icon and chart slots.

pub(crate) mod look;

pub use look::Trend;

use crate::layer::with_base_recipe_style;
use crate::stat::look::{StatState, StatTheme};
use kivact_core::custom::Caller;
use kivact_core::state::Value;
use kivact_core::view::{Role, View, node, text};
use kivact_macros::component;

/// A statistic: a label, a value, and optional change, icon and chart slots.
/// `label_content` and `value_content` replace the visible text and keep its
/// accessible name.
#[component]
pub fn stat(
    label: Value<String>,
    value: Value<String>,
    delta: Option<Value<String>>,
    #[prop(default = Trend::Neutral)] trend: Value<Trend>,
    icon: Option<View>,
    chart: Option<View>,
    label_content: Option<View>,
    value_content: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let state = move || StatState { trend: trend.get() };
    let variants = &variants;
    let wrap = |view: View, pick: fn(&StatTheme) -> &crate::recipe::Recipe<StatState>| {
        with_base_recipe_style(node().child(view), pick, variants, state)
    };
    let label_node = wrap(
        label_content.unwrap_or_else(|| text(label).into_view()),
        |r: &StatTheme| r.label_recipe(),
    )
    .name(label);
    let value_node = wrap(
        value_content.unwrap_or_else(|| text(value).into_view()),
        |r: &StatTheme| r.value_recipe(),
    )
    .name(value);
    let delta_node = delta.map(|v| wrap(text(v).into_view(), |r: &StatTheme| r.delta_recipe()));
    let icon_node = icon.map(|v| wrap(v, |r: &StatTheme| r.icon_recipe()));
    let chart_node = chart.map(|v| wrap(v, |r: &StatTheme| r.chart_recipe()));
    with_base_recipe_style(
        node().role(Role::Group).name(label).value_text(value),
        |r: &StatTheme| r.root_recipe(),
        variants,
        state,
    )
    .child((icon_node, label_node, value_node, delta_node, chart_node))
}
