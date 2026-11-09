//! Role: Apply theme recipe styles before caller overrides without extra nodes.
use super::recipes::{Recipe, UiRecipes};
use super::variants::Variants;
use kivact_core::style::{Theme, use_theme};
use kivact_core::view::NodeBuilder;
use std::rc::Rc;

/// The standard components' recipes of `theme`.
pub(super) fn recipes(theme: &Theme) -> kivact_core::style::ThemeRef<UiRecipes> {
    theme.ext()
}

/// `root` styled by the recipe `pick` chooses from the theme in effect,
/// given `state` and the component's `variants`, under the caller's later
/// styles.
pub(super) fn with_recipe_style<N: NodeBuilder, S: 'static>(
    root: N,
    pick: fn(&UiRecipes) -> &Recipe<S>,
    variants: &Variants,
    state: impl Fn() -> S + 'static,
) -> N {
    let variants = variants.clone();
    let name = pick(UiRecipes::default_table()).name();
    root.recipe_style(Rc::new(move || name), move || {
        let theme = use_theme().get();
        pick(&recipes(&theme))(&theme, &variants.look(state()))
    })
}

/// [`with_recipe_style`] for a builder whose state is known only once it is
/// complete (`into_view`): the recipe is the root's base style, below every
/// `.style(..)` of the caller whatever the order of the calls.
pub(super) fn with_base_recipe_style<N: NodeBuilder, S: 'static>(
    root: N,
    pick: fn(&UiRecipes) -> &Recipe<S>,
    variants: &Variants,
    state: impl Fn() -> S + 'static,
) -> N {
    let variants = variants.clone();
    let name = pick(UiRecipes::default_table()).name();
    root.with_node(|node| {
        node.base_recipe_style(Rc::new(move || name), move || {
            let theme = use_theme().get();
            pick(&recipes(&theme))(&theme, &variants.look(state()))
        })
    })
}
