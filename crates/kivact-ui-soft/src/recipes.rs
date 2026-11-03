//! Role: Cache the standard component recipe table for the Soft theme.
use kivact::ui::UiRecipes;
use std::sync::OnceLock;

pub(super) fn recipes() -> UiRecipes {
    static RECIPES: OnceLock<UiRecipes> = OnceLock::new();
    RECIPES
        .get_or_init(|| {
            crate::recipes_controls::register(UiRecipes::default())
                .text_input(|_| crate::recipes_field::text_input)
                .text_area(|_| crate::recipes_field::text_area)
                .field(crate::recipes_field::field)
                .number_input(crate::recipes_field::number_input)
                .tag_input(crate::recipes_field::tag_input)
                .select(crate::recipes_select::select)
                .menu(crate::recipes_menu::menu)
                .dialog(crate::recipes_dialog::dialog)
                .tooltip(crate::recipes_popup::tooltip)
                .toast(crate::recipes_popup::toast)
                .command_palette(crate::recipes_popup_command::command_palette)
                .tabs(crate::recipes_collections::tabs)
                .disclosure(crate::recipes_collections::disclosure)
                .splitter(crate::recipes_collections::splitter)
                .split(crate::recipes_collections::split)
                .list(crate::recipes_collections::list)
                .data_table(crate::recipes_collections::data_table)
        })
        .clone()
}
