//! Role: The number input recipe state, tokens, recipe table and default looks.
use crate::look_common::flat;
use crate::theme::Look;

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, TextAlign, Theme, style};
/// The decrease button, the input and the increase button in one box.
pub(crate) fn number_root(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .border(1, kivact_core::style::color::BORDER)
        .border_radius(t.get(&kivact_core::style::radius::MD_VAR))
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT)
        .clip(true)
}

/// The number, centered, without a box of its own.
pub(crate) fn number_input(t: &Theme, _: &Look<PartState>) -> Style {
    flat(t)
        .flex_grow(1)
        .min_width(40)
        .width(56)
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR) - 2.0)
        .text_align(TextAlign::Center)
}

recipe_table! {
    /// The recipes of a [`number_input`](super::number_input).
    NumberInputTheme {
        /// The box of the decrease button, the input and the increase button.
        root(root_recipe): PartState = number_root,
        /// The input, over the text input recipe.
        input(input_recipe): PartState = number_input,
    }
}
