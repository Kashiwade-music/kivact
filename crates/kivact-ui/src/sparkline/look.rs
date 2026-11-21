//! Role: The sparkline recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Style, Theme, pct, style};
pub(crate) fn sparkline_root(_: &Theme, _: &Look<()>) -> Style {
    style().width(pct(100)).height(32).min_width(1)
}

recipe_table! {
    /// The recipes of a [`sparkline`](super::sparkline).
    SparklineTheme {
        /// The chart's box: its size. The theme's `UiTokens::sparkline` draws the line and the area in it.
        root(root_recipe): () = sparkline_root,
    }
}
