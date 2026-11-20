//! Role: The empty state recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, Direction, Justify, Style, Theme, pct, style};
pub(crate) fn empty_root(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .direction(Direction::Column)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .padding(t.get(&kivact_core::style::space::XL_VAR))
        .width(pct(100))
        .min_width(0)
}

pub(crate) fn empty_title(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR) + 4.0)
        .weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
        .color(kivact_core::style::color::TEXT)
}

pub(crate) fn empty_description(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn empty_icon(_: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

pub(crate) fn empty_actions(t: &Theme, _: &Look<EmptyStateState>) -> Style {
    style()
        .direction(Direction::Row)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// The semantic state passed to every empty-state recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyStateState {
    pub heading_level: u8,
}

recipe_table! {
    /// Recipes for an empty region's message and actions.
    EmptyStateRecipes {
        root(root_recipe): EmptyStateState = empty_root,
        title(title_recipe): EmptyStateState = empty_title,
        description(description_recipe): EmptyStateState = empty_description,
        icon(icon_recipe): EmptyStateState = empty_icon,
        actions(actions_recipe): EmptyStateState = empty_actions,
    }
}
