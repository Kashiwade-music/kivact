//! Role: The async view's recipe table and its default look.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{FlexDirection, Style, Theme, style};

pub(crate) fn async_view_root(_: &Theme, _: &Look<()>) -> Style {
    style().flex_direction(FlexDirection::Column)
}

recipe_table! {
    /// The recipe of an async view: its root, which holds the content in a column.
    AsyncViewTheme {
        root(root_recipe): () = async_view_root,
    }
}
