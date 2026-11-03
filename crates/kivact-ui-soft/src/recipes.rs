//! Role: Cache the standard component recipe table for the Soft theme.
use kivact::ui::UiRecipes;
use std::sync::OnceLock;

pub(super) fn recipes() -> UiRecipes {
    static RECIPES: OnceLock<UiRecipes> = OnceLock::new();
    RECIPES.get_or_init(UiRecipes::default).clone()
}
