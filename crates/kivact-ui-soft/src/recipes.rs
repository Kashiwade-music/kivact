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
                .calendar(crate::recipes_calendar::calendar)
                .date_input(|r| {
                    r.root(|_| crate::recipes_calendar::field)
                        .wheel(|_| crate::recipes_calendar::wheel)
                })
                .date_range_input(|r| r.root(|_| crate::recipes_calendar::field))
                .time_input(|r| {
                    r.root(|_| crate::recipes_calendar::field)
                        .wheel(|_| crate::recipes_calendar::wheel)
                })
                .date_time_input(|r| r.root(|_| crate::recipes_calendar::field))
                .field(crate::recipes_field::field)
                .number_input(crate::recipes_field::number_input)
                .tag_input(crate::recipes_field::tag_input)
                .select(crate::recipes_select::select)
                .combobox(crate::recipes_combobox::combobox)
                .badge(|_| crate::recipes_badge::badge)
                .skeleton(|_| crate::recipes_skeleton::skeleton)
                .spinner(|_| crate::spinner::spinner)
                .kbd(|_| crate::kbd::root)
                .kbd_key(|_| crate::kbd::key)
                .button_group(crate::recipes_groups::button_group)
                .toggle_group(crate::recipes_groups::toggle_group)
                .toolbar(crate::recipes_groups::toolbar)
                .menu(crate::recipes_menu::menu)
                .dialog(crate::recipes_dialog::dialog)
                .hover_card(crate::recipes_popup::hover_card)
                .tooltip(crate::recipes_popup::tooltip)
                .toast(crate::recipes_popup::toast)
                .command_palette(crate::recipes_popup_command::command_palette)
                .tabs(crate::recipes_collections::tabs)
                .disclosure(crate::recipes_collections::disclosure)
                .splitter(crate::recipes_collections::splitter)
                .split(crate::recipes_collections::split)
                .list(crate::recipes_collections::list)
                .tree(crate::recipes_collections::tree)
                .data_table(crate::recipes_collections::data_table)
        })
        .clone()
}
