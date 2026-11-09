//! Role: Present a named statistic with optional delta, icon and chart slots.
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::view::{IntoView, Node, NodeBuilder, Role, View, node, text};

use crate::component_values::{StatState, Trend};
/// A statistic labelled by its visible label, created by [`stat`].
pub struct Stat {
    root: Node,
    label: Value<String>,
    value: Value<String>,
    // xtask-allow(state-fields): builder option - delta
    delta: Option<Value<String>>,
    trend: Value<Trend>,
    // xtask-allow(state-fields): builder option - icon
    icon: Option<View>,
    // xtask-allow(state-fields): builder option - chart
    chart: Option<View>,
    // xtask-allow(state-fields): builder option - label content
    label_content: Option<View>,
    // xtask-allow(state-fields): builder option - value content
    value_content: Option<View>,
    variants: Variants,
}
/// Show application-formatted text as a statistic. Reactive text updates in place.
#[track_caller]
pub fn stat(label: impl IntoText, value: impl IntoText) -> Stat {
    Stat {
        root: node().role(Role::Group),
        label: label.into_text_value(),
        value: value.into_text_value(),
        delta: None,
        trend: Trend::Neutral.into_value(),
        icon: None,
        chart: None,
        label_content: None,
        value_content: None,
        variants: Variants::default(),
    }
}
impl Stat {
    /// Show application-formatted change text; direction is set separately with [`Self::trend`].
    pub fn delta(mut self, value: impl IntoText) -> Self {
        self.delta = Some(value.into_text_value());
        self
    }
    /// Set the direction read by every recipe (neutral by default).
    pub fn trend(mut self, value: impl IntoValue<Trend>) -> Self {
        self.trend = value.into_value();
        self
    }
    /// Set the decorative icon or illustration.
    pub fn icon(mut self, value: impl IntoView) -> Self {
        self.icon = Some(value.into_view());
        self
    }
    /// Show an application-owned chart such as a sparkline.
    pub fn chart(mut self, value: impl IntoView) -> Self {
        self.chart = Some(value.into_view());
        self
    }
    /// Replace the visible label while retaining the accessible label text.
    pub fn label_content(mut self, value: impl IntoView) -> Self {
        self.label_content = Some(value.into_view());
        self
    }
    /// Replace the visible value while retaining its accessible text.
    pub fn value_content(mut self, value: impl IntoView) -> Self {
        self.value_content = Some(value.into_view());
        self
    }
    /// Add an appearance read by every recipe of the statistic.
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for Stat {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Stat {
    fn into_view(self) -> View {
        let trend = self.trend;
        let state = move || StatState { trend: trend.get() };
        let variants = &self.variants;
        let wrap = |view: View, pick: fn(&crate::recipes::UiRecipes) -> &crate::recipes::Recipe<StatState>| {
            with_base_recipe_style(node().child(view), pick, variants, state)
        };
        let label = wrap(
            self.label_content
                .unwrap_or_else(|| text(self.label).into_view()),
            |r| r.stat_recipes().label_recipe(),
        )
        .name(self.label);
        let value = wrap(
            self.value_content
                .unwrap_or_else(|| text(self.value).into_view()),
            |r| r.stat_recipes().value_recipe(),
        )
        .name(self.value);
        let delta = self
            .delta
            .map(|v| wrap(text(v).into_view(), |r| r.stat_recipes().delta_recipe()));
        let icon = self
            .icon
            .map(|v| wrap(v, |r| r.stat_recipes().icon_recipe()));
        let chart = self
            .chart
            .map(|v| wrap(v, |r| r.stat_recipes().chart_recipe()));
        with_base_recipe_style(
            self.root.name(self.label).value_text(self.value),
            |r| r.stat_recipes().root_recipe(),
            variants,
            state,
        )
        .child((icon, label, value, delta, chart))
        .into_view()
    }
}
