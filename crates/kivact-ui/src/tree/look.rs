//! Role: The tree recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
use kivact_core::view::{ReadingDirection, use_direction};
pub(crate) fn tree_item(t: &Theme, row: &TreeRowRecipeState) -> Style {
    let rtl = use_direction().get() == ReadingDirection::Rtl;
    let tokens = t.ext::<crate::tokens::UiTokens>();
    let tree = &tokens.tree;
    let padding = tree.padding.unwrap_or(t.space().md);
    let indent = padding + row.level.saturating_sub(1) as f64 * tree.indent.unwrap_or(16.0);
    style()
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical)
        .direction(if rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Center)
        .min_height(tree.row_height.unwrap_or(t.control_height()))
        .padding_left(if rtl { padding } else { indent })
        .padding_right(if rtl { indent } else { padding })
        .gap(tree.gap.unwrap_or(t.space().xs))
        .radius(tree.radius.unwrap_or(t.radius().sm))
        .hover(style().background(ColorToken::Hover))
        .selected(style().background(ColorToken::Selection))
}

pub(crate) fn tree_marker(t: &Theme, _: &TreeRowRecipeState) -> Style {
    style()
        .width(
            t.ext::<crate::tokens::UiTokens>()
                .tree
                .marker_width
                .unwrap_or(16.0),
        )
        .shrink(0)
}

/// The mounted row's interaction state and structural position in its visible tree.
#[derive(Clone, PartialEq)]
pub struct TreeRowRecipeState {
    pub item: kivact_interaction::TreeItemState,
    /// Whether each ancestor is the last visible sibling at its level.
    pub ancestor_last: Vec<bool>,
    pub is_last: bool,
    pub has_branches: bool,
    /// The caller supplied marker content instead of the standard glyph.
    pub custom_marker: bool,
}

impl std::ops::Deref for TreeRowRecipeState {
    type Target = kivact_interaction::TreeItemState;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

recipe_table! {
    /// The recipes of a [`tree`](super::tree).
    TreeRecipes {
        /// The column containing the visible rows.
        root(root_recipe): PartState = crate::look_common::list,
        /// A row with the expansion indicator and its content.
        item(item_recipe): TreeRowRecipeState = tree_item,
        /// The branch indicator or matching leaf spacer.
        marker(marker_recipe): TreeRowRecipeState = tree_marker,
    }
}
