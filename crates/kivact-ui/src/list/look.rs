//! Role: The list recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
use kivact_interaction::ListItemState;
/// An item: its icon beside its label, the whole row marked while it is
/// selected.
pub(crate) fn list_item(t: &Theme, _: &ListItemState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .min_height(t.control_height())
        .padding_x(t.space().md)
        .radius(t.radius().sm)
        .color(ColorToken::Text)
        .hover(style().background(ColorToken::Hover))
        .selected(style().background(ColorToken::Selection))
}

/// The secondary line of an item with a description.
pub(crate) fn list_description(t: &Theme, _: &ListItemState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

recipe_table! {
    /// The recipes of a [`list`](super::list).
    ListRecipes {
        /// The column of items.
        root(root_recipe): PartState = crate::look_common::list,
        /// An item row: its icon, its label and its description.
        item(item_recipe): ListItemState = list_item,
        /// The secondary line of an item with a description.
        description(description_recipe): ListItemState = list_description,
    }
}
