//! Role: The description list recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
pub(crate) fn description_root(t: &Theme, _: &Look<()>) -> Style {
    style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .min_width(0)
}

pub(crate) fn description_item(t: &Theme, _: &Look<()>) -> Style {
    style()
        .direction(Direction::Row)
        .gap(t.space().md)
        .align_items(Align::Center)
        .min_width(0)
}

pub(crate) fn description_term(t: &Theme, _: &Look<()>) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
        .shrink(0)
}

pub(crate) fn description_detail(_: &Theme, _: &Look<()>) -> Style {
    style().grow(1).min_width(0)
}

recipe_table! {
    /// The list, pair, visible term and labelled detail of a description list.
    DescriptionListRecipes {
        root(root_recipe): () = description_root,
        item(item_recipe): () = description_item,
        term(term_recipe): () = description_term,
        detail(detail_recipe): () = description_detail,
    }
}
