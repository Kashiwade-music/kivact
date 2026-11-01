//! Role: Apply theme recipe styles before caller overrides without extra nodes.
use super::recipes::{Recipe, UiRecipes};
use super::variants::Variants;
use kivact_core::control::NodeBuilder;
use kivact_core::primitive::NodeExt;
use kivact_core::style::{Theme, current_theme};
use std::sync::Arc;

/// The standard components' recipes of `theme`.
pub(super) fn recipes(theme: &Theme) -> Arc<UiRecipes> {
    theme.get()
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
    root.style(move || {
        let theme = current_theme();
        pick(&recipes(&theme))(&theme, &variants.look(state()))
    })
}

/// [`with_recipe_style`] for a builder whose state is known only once it is
/// complete (`into_view`): the recipe is the root's base style, below every
/// `.style(..)` of the caller whatever the order of the calls.
pub(super) fn with_base_recipe_style<N: NodeExt, S: 'static>(
    root: N,
    pick: fn(&UiRecipes) -> &Recipe<S>,
    variants: &Variants,
    state: impl Fn() -> S + 'static,
) -> N {
    let variants = variants.clone();
    root.base_style(move || {
        let theme = current_theme();
        pick(&recipes(&theme))(&theme, &variants.look(state()))
    })
}
