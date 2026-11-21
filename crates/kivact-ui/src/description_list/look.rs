//! Role: The description list recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
pub(crate) fn description_root(t: &Theme, _: &Look<()>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .min_width(0)
}

pub(crate) fn description_item(t: &Theme, _: &Look<()>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .align_items(Align::Center)
        .min_width(0)
}

pub(crate) fn description_term(t: &Theme, _: &Look<()>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
        .flex_shrink(0)
}

pub(crate) fn description_detail(_: &Theme, _: &Look<()>) -> Style {
    style().flex_grow(1).min_width(0)
}

recipe_table! {
    /// The list, pair, visible term and labelled detail of a description list.
    DescriptionListTheme {
        root(root_recipe): () = description_root,
        item(item_recipe): () = description_item,
        term(term_recipe): () = description_term,
        detail(detail_recipe): () = description_detail,
    }
}
