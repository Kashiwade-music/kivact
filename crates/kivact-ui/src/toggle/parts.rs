//! Role: Mount persistent themeable indicator parts for checkbox and switch motion.
use crate::layer::with_recipe_style;
use crate::toggle::look::{CheckableState, CheckboxTheme, SwitchTheme};
use kivact_core::state::Value;
use kivact_core::style::{Position, style, use_theme};
use kivact_core::view::Variants;
use kivact_core::view::{IntoView, show};
use kivact_core::view::{Node, node};

pub(super) fn checkbox(variants: &Variants, state: Value<CheckableState>) -> impl IntoView {
    let variants = variants.clone();
    show(
        || use_theme().get().ext::<CheckboxTheme>().has_parts(),
        move || checkbox_parts(&variants, state),
    )
}

fn checkbox_parts(variants: &Variants, state: Value<CheckableState>) -> Node {
    with_recipe_style(
        node().style(style().position(Position::Absolute)),
        |r: &CheckboxTheme| r.indicator_recipe(),
        variants,
        move || state.get(),
    )
    .child((
        with_recipe_style(
            node(),
            |r: &CheckboxTheme| r.check_recipe(),
            variants,
            move || state.get(),
        ),
        with_recipe_style(
            node(),
            |r: &CheckboxTheme| r.mixed_recipe(),
            variants,
            move || state.get(),
        ),
    ))
}

pub(super) fn switch(variants: &Variants, state: Value<CheckableState>) -> impl IntoView {
    let variants = variants.clone();
    show(
        || use_theme().get().ext::<SwitchTheme>().has_parts(),
        move || switch_parts(&variants, state),
    )
}

fn switch_parts(variants: &Variants, state: Value<CheckableState>) -> Node {
    with_recipe_style(
        node().style(style().position(Position::Absolute)),
        |r: &SwitchTheme| r.track_recipe(),
        variants,
        move || state.get(),
    )
    .child(with_recipe_style(
        node(),
        |r: &SwitchTheme| r.thumb_recipe(),
        variants,
        move || state.get(),
    ))
}
