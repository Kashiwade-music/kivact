//! Role: The tag input recipe state, tokens, recipe table and default looks.
use crate::look_common::flat;

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, Color, ColorToken, Direction, Style, Theme, style};
/// The box of the tags and the input, ringed while the focus is inside.
pub(crate) fn tag_root(t: &Theme, state: &PartState) -> Style {
    let s = style()
        .direction(Direction::Row)
        .wrap(true)
        .align_items(Align::Center)
        .gap(t.space().sm)
        .min_height(t.control_height())
        .padding(t.space().sm)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
        .color(ColorToken::Text);
    if state.is_focused() {
        s.border_color(ColorToken::Focus).ring_with_offset(
            t.focus_ring(),
            t.focus_ring_offset(),
            Color::token(ColorToken::Focus).with_alpha(0.35),
        )
    } else {
        s
    }
}

/// One tag: its text and its remove button.
pub(crate) fn tag(t: &Theme, _: &PartState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().xs)
        .padding_left(t.space().md)
        .padding_right(t.space().xs)
        .min_height(t.control_height() - 10.0)
        .radius(t.radius().sm)
        .background(ColorToken::Selection)
        .font_size(t.text().size_small)
}

/// A tag's remove button, small enough to sit in the tag.
pub(crate) fn tag_remove(t: &Theme, _: &PartState) -> Style {
    style().size(18, 18).padding(0).radius(t.radius().sm)
}

/// The input among the tags, without a box of its own: the root's is the
/// field's.
pub(crate) fn tag_input(t: &Theme, _: &PartState) -> Style {
    flat(t)
        .grow(1)
        .min_width(96)
        .height(t.control_height() - 10.0)
}

recipe_table! {
    /// The recipes of a [`tag_input`](super::tag_input).
    TagInputRecipes {
        /// The box of the tags and the input; its `focused` is true while the
        /// focus is in the input.
        root(root_recipe): PartState = tag_root,
        /// One tag: its text and its remove button.
        tag(tag_recipe): PartState = tag,
        /// A tag's remove button, over the button recipes.
        remove(remove_recipe): PartState = tag_remove,
        /// The input, over the text input recipe.
        input(input_recipe): PartState = tag_input,
    }
}
