//! Role: The number input recipe state, tokens, recipe table and default looks.
use crate::look_common::flat;

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, ColorToken, Direction, Style, TextAlign, Theme, style};
/// The decrease button, the input and the increase button in one box.
pub(crate) fn number_root(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .height(t.control_height())
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .clip(true)
}

/// The number, centered, without a box of its own.
pub(crate) fn number_input(t: &Theme, _: &PartState) -> Style {
    flat(t)
        .grow(1)
        .min_width(40)
        .width(56)
        .height(t.control_height() - 2.0)
        .text_align(TextAlign::Center)
}

recipe_table! {
    /// The recipes of a [`number_input`](super::number_input).
    NumberInputRecipes {
        /// The box of the decrease button, the input and the increase button.
        root(root_recipe): PartState = number_root,
        /// The input, over the text input recipe.
        input(input_recipe): PartState = number_input,
    }
}
