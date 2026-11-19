//! Role: The avatar group recipe state, tokens, recipe table and default looks.

use kivact_core::style::{Align, Style, Theme, style};
pub(crate) fn avatar_group(_: &Theme, _: &()) -> Style {
    style().align_items(Align::Center)
}

pub(crate) fn avatar_group_member(_: &Theme, _: &usize) -> Style {
    style()
}

pub(crate) fn avatar_group_overflow(_: &Theme, _: &usize) -> Style {
    style()
}
