//! Role: The list recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
use kivact_interaction::ListItemState;
/// An item: its icon beside its label, the whole row marked while it is
/// selected.
pub(crate) fn list_item(t: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .min_height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .color(kivact_core::style::color::TEXT)
        .hover(style().background(kivact_core::style::color::HOVER))
        .selected(style().background(kivact_core::style::color::SELECTION))
}

/// The secondary line of an item with a description.
pub(crate) fn list_description(t: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

recipe_table! {
    /// The recipes of a [`list`](super::list).
    ListTheme {
        /// The column of items.
        root(root_recipe): PartState = crate::look_common::list,
        /// An item row: its icon, its label and its description.
        item(item_recipe): ListItemState = list_item,
        /// The secondary line of an item with a description.
        description(description_recipe): ListItemState = list_description,
    }
}
