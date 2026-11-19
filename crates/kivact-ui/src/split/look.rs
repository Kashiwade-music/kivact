//! Role: The split recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::slider::look::AxisState;
use kivact_core::style::{Align, ColorToken, Style, Theme, style};
use kivact_core::view::Orientation;
/// The grip on a split's handle: a short rounded bar across it.
pub(crate) fn split_grip(_: &Theme, state: &AxisState) -> Style {
    let s = style()
        .shrink(0)
        .align_self(Align::Center)
        .radius(2)
        .background(ColorToken::Border);
    match state.orientation {
        Orientation::Vertical => s.size(24, 4),
        _ => s.size(4, 24),
    }
}

recipe_table! {
    /// The recipes of a [`split`](super::split), besides its handles'
    /// [`SplitterRecipes::handle`].
    SplitRecipes {
        /// The grip on a handle, shown while [`Split::grip`](super::Split::grip)
        /// is true; its state is the handle's.
        grip(grip_recipe): AxisState = split_grip,
    }
}
