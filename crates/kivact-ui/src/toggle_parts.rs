//! Role: Mount persistent themeable indicator parts for checkbox and switch motion.
use super::layer::{recipes, with_recipe_style};
use super::values::CheckableState;
use super::variants::Variants;
use kivact_core::state::Value;
use kivact_core::style::{Position, style, use_theme};
use kivact_core::view::{IntoView, show};
use kivact_core::view::{Node, node};

pub(super) fn checkbox(variants: &Variants, state: Value<CheckableState>) -> impl IntoView {
    let variants = variants.clone();
    show(
        || recipes(&use_theme().get()).has_checkbox_parts(),
        move || checkbox_parts(&variants, state),
    )
}

fn checkbox_parts(variants: &Variants, state: Value<CheckableState>) -> Node {
    with_recipe_style(
        node().style(style().position(Position::Absolute)),
        |r| r.checkbox_indicator_recipe(),
        variants,
        move || state.get(),
    )
    .child((
        with_recipe_style(
            node(),
            |r| r.checkbox_check_recipe(),
            variants,
            move || state.get(),
        ),
        with_recipe_style(
            node(),
            |r| r.checkbox_mixed_recipe(),
            variants,
            move || state.get(),
        ),
    ))
}

pub(super) fn switch(variants: &Variants, state: Value<CheckableState>) -> impl IntoView {
    let variants = variants.clone();
    show(
        || recipes(&use_theme().get()).has_switch_parts(),
        move || switch_parts(&variants, state),
    )
}

fn switch_parts(variants: &Variants, state: Value<CheckableState>) -> Node {
    with_recipe_style(
        node().style(style().position(Position::Absolute)),
        |r| r.switch_track_recipe(),
        variants,
        move || state.get(),
    )
    .child(with_recipe_style(
        node(),
        |r| r.switch_thumb_recipe(),
        variants,
        move || state.get(),
    ))
}
