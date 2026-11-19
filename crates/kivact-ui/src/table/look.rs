//! Role: The table recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::tokens::UiTokens;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, pct, style};
pub(crate) fn table(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .width(pct(100))
        .min_width(0)
        .font_size(
            t.ext::<UiTokens>()
                .table
                .font_size
                .unwrap_or(t.text().size_small),
        )
}

pub(crate) fn group(_: &Theme, _: &PartState) -> Style {
    style().direction(Direction::Column)
}

pub(crate) fn footer(_: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Column)
        .background(ColorToken::SurfaceAlt)
}

pub(crate) fn row(_: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .width(pct(100))
        .align_items(Align::Center)
        .border_bottom(1, ColorToken::Border)
        .selected(style().background(ColorToken::Selection))
}

pub(crate) fn cell(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_width(0)
        .padding(
            t.ext::<UiTokens>()
                .table
                .cell_padding
                .unwrap_or(t.space().sm),
        )
        .color(ColorToken::Text)
}

pub(crate) fn head(t: &Theme, s: &PartState) -> Style {
    cell(t, s)
        .min_height(
            t.ext::<UiTokens>()
                .table
                .header_height
                .unwrap_or(t.control_height()),
        )
        .weight(600)
}

recipe_table! {
    /// Static table part recipes.
    TableRecipes {
        root(root_recipe): PartState = table,
        header(header_recipe): PartState = group,
        body(body_recipe): PartState = group,
        footer(footer_recipe): PartState = footer,
        row(row_recipe): PartState = row,
        head(head_recipe): PartState = head,
        cell(cell_recipe): PartState = cell,
    }
}
