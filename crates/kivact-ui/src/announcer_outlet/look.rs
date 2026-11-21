//! Role: The announcer outlet's recipe table and its default look.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{Position, Style, Theme, style};

pub(crate) fn announcer_outlet_root(_: &Theme, _: &Look<()>) -> Style {
    style()
        .position(Position::Absolute)
        .left(0)
        .top(0)
        .size(0, 0)
}

recipe_table! {
    /// The recipe of an announcer outlet: its root, outside the layout.
    AnnouncerOutletTheme {
        root(root_recipe): () = announcer_outlet_root,
    }
}
