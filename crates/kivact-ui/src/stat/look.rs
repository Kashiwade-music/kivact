//! Role: The stat recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Direction, Style, Theme, style};
use kivact_macros::Prop;
pub(crate) fn stat_root(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .padding(t.get(&kivact_core::style::space::MD_VAR))
        .min_width(0)
        .background(kivact_core::style::color::SURFACE)
        .border(1, kivact_core::style::color::BORDER)
        .radius(t.get(&kivact_core::style::radius::MD_VAR))
}

pub(crate) fn stat_label(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn stat_value(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR) * 2.0)
        .weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn stat_delta(t: &Theme, _: &Look<StatState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
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
