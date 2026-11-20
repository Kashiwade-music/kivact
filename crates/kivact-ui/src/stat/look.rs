//! Role: The stat recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{ColorToken, Direction, Style, Theme, style};
use kivact_macros::Prop;
pub(crate) fn stat_root(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .padding(t.space().md)
        .min_width(0)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
}

pub(crate) fn stat_label(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

pub(crate) fn stat_value(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.text().size * 2.0)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
}

pub(crate) fn stat_delta(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

pub(crate) fn stat_slot(_: &Theme, _: &Look<StatState>) -> Style {
    style().min_width(0)
}

/// Semantic state shared by all statistic recipes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatState {
    /// Direction supplied by the application; it does not imply a success or danger tone.
    pub trend: Trend,
}

recipe_table! {
    /// Parts of a statistic and its optional application-owned chart.
    StatRecipes {
        root(root_recipe): StatState = stat_root,
        label(label_recipe): StatState = stat_label,
        value(value_recipe): StatState = stat_value,
        delta(delta_recipe): StatState = stat_delta,
        icon(icon_recipe): StatState = stat_slot,
        chart(chart_recipe): StatState = stat_slot,
    }
}

/// The direction of change, independent of whether that change is desirable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum Trend {
    /// The statistic has increased.
    Up,
    /// The statistic has decreased.
    Down,
    /// The statistic is steady or has no stated direction.
    #[default]
    Neutral,
}
