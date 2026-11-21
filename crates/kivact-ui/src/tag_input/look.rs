//! Role: The tag input recipe state, tokens, recipe table and default looks.
use crate::look_common::flat;

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
/// The box of the tags and the input, ringed while the focus is inside.
pub(crate) fn tag_root(t: &Theme, state: &PartState) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Row)
        .flex_wrap(true)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .min_height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .padding(t.get(&kivact_core::style::space::SM_VAR))
        .border(1, kivact_core::style::color::BORDER)
        .border_radius(t.get(&kivact_core::style::radius::MD_VAR))
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT);
    if state.is_focused() {
        s.border_color(kivact_core::style::color::FOCUS)
            .outline(
                t.focus_ring(),
                kivact_core::style::color::FOCUS.with_alpha(0.35),
            )
            .outline_offset(t.focus_ring_offset())
    } else {
        s
    }
}

/// One tag: its text and its remove button.
pub(crate) fn tag(t: &Theme, _: &PartState) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(t.get(&kivact_core::style::space::XS_VAR))
        .padding_left(t.get(&kivact_core::style::space::MD_VAR))
        .padding_right(t.get(&kivact_core::style::space::XS_VAR))
        .min_height(t.get(&kivact_core::style::control::HEIGHT_VAR) - 10.0)
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .background(kivact_core::style::color::SELECTION)
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
}

/// A tag's remove button, small enough to sit in the tag.
pub(crate) fn tag_remove(t: &Theme, _: &PartState) -> Style {
    style()
        .size(18, 18)
        .padding(0)
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
}

/// The input among the tags, without a box of its own: the root's is the
/// field's.
pub(crate) fn tag_input(t: &Theme, _: &PartState) -> Style {
    flat(t)
        .flex_grow(1)
        .min_width(96)
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR) - 10.0)
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
