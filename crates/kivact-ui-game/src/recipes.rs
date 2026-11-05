//! Role: Install the immutable Game recipe table.
use crate::recipes_controls::{
    button, button_busy, button_content, checkbox, switch, text_area, text_input, toggle,
};
use kivact::style::Align;
use kivact::ui::UiRecipes;

/// The Game recipe table, built once and shared, so two themes of the kit
/// compare their recipes by identity and installing an unchanged theme
/// never restyles.
pub(crate) fn recipes() -> UiRecipes {
    #[expect(
        clippy::disallowed_types,
        reason = "Immutable static recipe identity is shared by all Game themes."
    )]
    static TABLE: std::sync::OnceLock<UiRecipes> = std::sync::OnceLock::new();
    TABLE.get_or_init(build).clone()
}

fn build() -> UiRecipes {
    install_controls(
        UiRecipes::default()
            .field(crate::recipes_field::field)
            .heading(|_| crate::recipes_metrics::heading)
            .stat(crate::recipes_metrics::stat)
            .description_list(crate::recipes_metrics::description_list)
            .skeleton(|_| crate::recipes_indicators::skeleton)
            .empty_state(crate::recipes_feedback::empty_state)
            .alert(crate::recipes_feedback::alert)
            .badge(|_| crate::recipes_badge::badge)
            .badge_label(|_| crate::recipes_badge::badge_label)
            .avatar(|_| crate::recipes_avatar::avatar)
            .avatar_status(|_| crate::recipes_avatar::avatar_status)
            .avatar_group(|base| move |t, s| base(t, s).align_items(Align::Center))
            .avatar_group_overflow(|_| crate::recipes_avatar::avatar_overflow)
            .tree(crate::recipes_items::tree)
            .table(crate::recipes_items::table)
            .item_list(crate::recipes_items::item_list)
            .dropzone(crate::recipes_files::dropzone)
            .toolbar(crate::recipes_groups::toolbar)
            .pagination(crate::recipes_navigation::pagination)
            .breadcrumb(crate::recipes_navigation::breadcrumb)
            .stepper(crate::recipes_navigation::stepper)
            .navigation_menu(crate::recipes_navigation::navigation_menu)
            .button_group(crate::recipes_groups::button_group)
            .toggle_group(crate::recipes_groups::toggle_group)
            .input_group(crate::recipes_groups::input_group)
            .button(|_| button)
            .button_content(|_| button_content)
            .button_busy(|_| button_busy)
            .toggle(|_| toggle)
            .checkbox(|_| checkbox)
            .switch(|_| switch)
            .text_input(|_| text_input)
            .text_area(|_| text_area),
    )
}

fn install_controls(recipes: UiRecipes) -> UiRecipes {
    recipes
        .select(|select| {
            select
                .trigger(|_| crate::recipes_extra::select_trigger)
                .surface(|_| crate::recipes_extra::select_surface)
                .option(|_| crate::recipes_extra::select_option)
                .check(|_| crate::recipes_extra::select_check)
                .indicator(|_| crate::recipes_extra::select_indicator)
        })
        .menu(|menu| {
            menu.trigger(|_| crate::recipes_extra::menu_trigger)
                .surface(|_| crate::recipes_extra::menu_surface)
                .item(|_| crate::recipes_extra::menu_item)
                .shortcut(|_| crate::recipes_extra::menu_shortcut)
                .separator(|_| crate::recipes_extra::menu_separator)
        })
        .dialog(|dialog| {
            dialog
                .surface(|_| crate::recipes_extra::dialog_surface)
                .title(|_| crate::recipes_extra::dialog_title)
                .description(|_| crate::recipes_extra::dialog_description)
                .header(|_| crate::recipes_extra::dialog_header)
                .body(|_| crate::recipes_extra::dialog_body)
                .footer(|_| crate::recipes_extra::dialog_footer)
                .scrim(|_| crate::recipes_extra::dialog_scrim)
                .close(|_| crate::recipes_extra::dialog_close)
        })
        .tooltip(|tooltip| tooltip.surface(|_| crate::recipes_extra::tooltip_surface))
        .tabs(|tabs| {
            tabs.list(|_| crate::recipes_extra::tabs_list)
                .tab(|_| crate::recipes_extra::tab)
        })
        .radio_group(|radio| {
            radio
                .group(|_| crate::recipes_extra::radio_group)
                .item(|_| crate::recipes_extra::radio_item)
                .indicator(|_| crate::recipes_extra::radio_indicator)
                .description(|_| crate::recipes_extra::radio_description)
        })
        .slider(|slider| {
            slider
                .track(|_| crate::recipes_extra::slider_track)
                .range(|_| crate::recipes_extra::slider_range)
                .thumb(|_| crate::recipes_extra::slider_thumb)
        })
        .toast(|toast| {
            toast
                .region(|_| crate::recipes_extra::toast_region)
                .toast(|_| crate::recipes_extra::toast_card)
                .title(|_| crate::recipes_extra::toast_title)
                .body(|_| crate::recipes_extra::toast_body)
                .icon(|_| crate::recipes_extra::toast_icon)
                .close(|_| crate::recipes_extra::toast_close)
        })
        .splitter(|splitter| splitter.handle(|_| crate::recipes_extra::splitter_handle))
        .list(crate::recipes_collections::list)
        .data_table(crate::recipes_collections::data_table)
        .progress(crate::recipes_small::progress)
        .disclosure(crate::recipes_small::disclosure)
        .command_palette(crate::recipes_palette::command_palette)
        .tag_input(crate::recipes_input::tag_input)
        .number_input(crate::recipes_input::number_input)
}
