//! Role: The pagination recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
pub(crate) fn root<S>(t: &Theme, _: &Look<S>) -> Style {
    let p = &t.ext::<crate::tokens::UiTokens>().pagination;
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(p.gap)
        .padding(p.padding)
        .font_size(t.text().size)
        .color(ColorToken::Text)
}

pub(crate) fn pagination_item(t: &Theme, s: &Look<PaginationItemState>) -> Style {
    style()
        .color(if s.current {
            t.colors().text
        } else {
            t.colors().text_muted
        })
        .weight(if s.current { 600 } else { 400 })
}

pub(crate) fn pagination_wrapper(_: &Theme, _: &Look<PaginationSummary>) -> Style {
    style().display(kivact_core::style::Display::Contents)
}

/// Actual page cell and selection supplied to the cell recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationItemState {
    pub current: bool,
    pub item: PageItem,
    pub mode: PaginationMode,
}

/// Actual inputs for a caller's pagination summary; no duplicate range endpoints are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationSummary {
    /// Zero-based actual page index.
    pub page: usize,
    pub count: usize,
    pub total_items: Option<u64>,
    pub page_size: u32,
    pub has_more: bool,
}

recipe_table! {
    /// Page control and gap recipes.
    PaginationRecipes {
        root(root_recipe): () = root,
        item(item_recipe): PaginationItemState = pagination_item,
        ellipsis(ellipsis_recipe): () = crate::look_common::part,
        dots(dots_recipe): () = crate::look_common::part,
        controls(controls_recipe): PaginationSummary = pagination_wrapper,
        sizes(sizes_recipe): PaginationSummary = pagination_wrapper,
        input(input_recipe): PaginationSummary = pagination_wrapper,
        summary(summary_recipe): PaginationSummary = crate::look_common::part,
    }
}

/// A cell in a pagination bar; page indices are zero based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageItem {
    First,
    Previous,
    Page(usize),
    Ellipsis,
    Next,
    Last,
}

/// Structural pagination presentations, including an editable page number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationMode {
    #[default]
    Pages,
    Count,
    Compact,
    Dots,
    Input,
    None,
}
