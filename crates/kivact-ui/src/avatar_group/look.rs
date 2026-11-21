//! Role: The avatar group recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{Align, Style, Theme, style};
pub(crate) fn avatar_group(_: &Theme, _: &Look<()>) -> Style {
    style().align_items(Align::Center)
}

pub(crate) fn avatar_group_member(_: &Theme, _: &Look<usize>) -> Style {
    style()
}

pub(crate) fn avatar_group_overflow(_: &Theme, _: &Look<usize>) -> Style {
    style()
}

recipe_table! {
    /// The recipes of an avatar group: its root, its members and its overflow count.
    AvatarGroupTheme {
        root(root_recipe): () = avatar_group,
        member(member_recipe): usize = avatar_group_member,
        /// Presentation of the hidden avatar count, including its actual count.
        overflow(overflow_recipe): usize = avatar_group_overflow,
    }
}
