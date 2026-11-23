//! Role: Apply theme recipe styles before caller overrides without extra nodes.
use crate::recipe::Recipe;
use kivact_core::style::{ThemeExtension, use_theme};
use kivact_core::view::Variants;
use kivact_core::view::{Element, Node};
use std::rc::Rc;

/// The looks a node carries from its authors, with the `inherited` looks of its
/// group under them: an inherited value of a type wins over the node's own.
pub(crate) fn own_and_inherited(root: &Node, inherited: &Variants) -> Variants {
    let mut looks = root.authored().variants();
    looks.merge(inherited);
    looks
}

/// `root` styled by the recipe `pick` chooses from the component's theme
/// table `T` in the theme in effect, given `state` and the component's
/// `variants`, under the caller's later styles.
pub(crate) fn with_recipe_style<N: Element, T: ThemeExtension + Default, S: 'static>(
    root: N,
    pick: fn(&T) -> &Recipe<S>,
    variants: &Variants,
    state: impl Fn() -> S + 'static,
) -> N {
    let variants = variants.clone();
    let name = pick(&T::default()).name();
    root.recipe_style(Rc::new(move || name), move || {
        let theme = use_theme().get();
        pick(&theme.ext::<T>()).style(&theme, &crate::recipe::Look::of(state(), variants.clone()))
    })
}

/// [`with_recipe_style`] for a builder whose state is known only once it
/// is complete: the recipe is the root's base style, below every
/// `.style(..)` of the caller whatever the order of the calls.
pub(crate) fn with_base_recipe_style<N: Element, T: ThemeExtension + Default, S: 'static>(
    root: N,
    pick: fn(&T) -> &Recipe<S>,
    variants: &Variants,
    state: impl Fn() -> S + 'static,
) -> N {
    let variants = variants.clone();
    let name = pick(&T::default()).name();
    root.with_node(|node| {
        node.base_recipe_style(Rc::new(move || name), move || {
            let theme = use_theme().get();
            pick(&theme.ext::<T>())
                .style(&theme, &crate::recipe::Look::of(state(), variants.clone()))
        })
    })
}
