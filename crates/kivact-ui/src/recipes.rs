use crate::alert::look::AlertRecipes;
use crate::breadcrumb::look::BreadcrumbRecipes;
use crate::calendar::look::CalendarRecipes;
use crate::card::look::CardRecipes;
use crate::carousel::look::CarouselRecipes;
use crate::chat_composer::look::ChatComposerRecipes;
use crate::chat_message::look::ChatMessageRecipes;
use crate::chat_thread::look::ChatThreadRecipes;
use crate::code_block::look::CodeBlockRecipes;
use crate::combobox::look::ComboboxRecipes;
use crate::command_palette::look::CommandPaletteRecipes;
use crate::data_table::look::DataTableRecipes;
use crate::date_input::look::DateInputRecipes;
use crate::date_range_input::look::DateRangeInputRecipes;
use crate::date_time_input::look::DateTimeInputRecipes;
use crate::description_list::look::DescriptionListRecipes;
use crate::dialog::look::DialogRecipes;
use crate::disclosure::look::DisclosureRecipes;
use crate::empty_state::look::EmptyStateRecipes;
use crate::hover_card::look::HoverCardRecipes;
use crate::item_list::look::ItemListRecipes;
use crate::list::look::ListRecipes;
use crate::menu::look::MenuRecipes;
use crate::navigation_menu::look::NavigationMenuRecipes;
use crate::pagination::look::PaginationRecipes;
use crate::progress::look::ProgressRecipes;
use crate::radio_group::look::RadioGroupRecipes;
use crate::select::look::SelectRecipes;
use crate::sparkline::look::SparklineRecipes;
use crate::stat::look::StatRecipes;
use crate::stepper::look::StepperRecipes;
use crate::table::look::TableRecipes;
use crate::tabs::look::TabsRecipes;
use crate::time_input::look::TimeInputRecipes;
use crate::toast::look::ToastRecipes;
use crate::tooltip::look::TooltipRecipes;
use crate::tree::look::TreeRecipes;
use crate::typing_indicator::look::TypingIndicatorRecipes;

use crate::button::look::ButtonState;
use crate::button_group::look::ButtonGroupRecipes;
use crate::dropzone::look::DropzoneRecipes;
use crate::field::look::FieldRecipes;
use crate::heading::look::HeadingState;
use crate::input_group::look::InputGroupRecipes;
use crate::number_input::look::NumberInputRecipes;
use crate::recipe::recipe_table;
use crate::slider::look::SliderRecipes;
use crate::split::look::SplitRecipes;
use crate::splitter::look::SplitterRecipes;
use crate::tag_input::look::TagInputRecipes;
use crate::text_input::look::TextInputState;
use crate::toggle::look::CheckableState;
use crate::toggle_button::look::ToggleButtonState;
use crate::toggle_group::look::ToggleGroupRecipes;
use crate::toolbar::look::ToolbarRecipes;

use kivact_core::style::{Theme, ThemeExtension};

recipe_table! {
    /// The recipes of every standard component, installed in a theme with
    /// [`Theme::with`]; a theme without them draws
    /// [`UiRecipes::default`].
    ///
    /// Each method replaces one recipe or one component's table with what
    /// its function makes of the current one, so a kit's theme replaces
    /// everything and an application changes one part of a kit's look:
    ///
    /// ```
    /// use kivact::style::Theme;
    /// use kivact::ui::UiRecipes;
    /// let theme = Theme::light();
    /// let recipes = theme.ext::<UiRecipes>();
    /// let square = recipes.to_owned()
    ///     .button(|prev| move |theme, state| prev(theme, state).radius(0))
    ///     .menu(|menu| menu.item(|prev| move |theme, state| prev(theme, state).padding_x(16)));
    /// let theme = theme.with(square);
    /// # let _ = theme;
    /// ```
    UiRecipes {
        /// Typography for a semantic heading.
        heading(heading_recipe): HeadingState = crate::heading::look::heading,
        avatar_status_content(avatar_status_content_recipe): crate::avatar::look::AvatarState = crate::avatar::look::avatar_status_content,
        avatar_status(avatar_status_recipe): crate::avatar::look::AvatarState = crate::avatar::look::avatar_status,
        avatar_group(avatar_group_recipe): () = crate::avatar_group::look::avatar_group,
        avatar_group_member(avatar_group_member_recipe): usize = crate::avatar_group::look::avatar_group_member,
        /// Presentation of the hidden avatar count, including its actual count.
        avatar_group_overflow(avatar_group_overflow_recipe): usize = crate::avatar_group::look::avatar_group_overflow,
        avatar(avatar_recipe): crate::avatar::look::AvatarState = crate::avatar::look::avatar,
        avatar_image(avatar_image_recipe): crate::avatar::look::AvatarState = crate::avatar::look::avatar_image,
        kbd(kbd_recipe): () = crate::kbd::look::kbd,
        kbd_key(kbd_key_recipe): crate::kbd::look::KbdKeyState = crate::kbd::look::kbd_key,
        badge(badge_recipe): crate::badge::look::BadgeState = crate::badge::look::badge,
        badge_label(badge_label_recipe): crate::badge::look::BadgeState = crate::badge::look::badge_label,
        spinner(spinner_recipe): crate::spinner::look::SpinnerState = crate::spinner::look::spinner,
        skeleton(skeleton_recipe): crate::skeleton::look::SkeletonState = crate::skeleton::look::skeleton,
        button(button_recipe): ButtonState = crate::button::look::button,
        /// The row of a button's icons, label and children.
        button_content(button_content_recipe): ButtonState = crate::button::look::button_content,
        /// The busy indicator while `busy`; the button turns it about
        /// its centre.
        button_busy(button_busy_recipe): ButtonState = crate::button::look::button_busy,
        /// The box around a button's leading and trailing icon: where a kit
        /// sizes the icon for the button's size.
        button_icon(button_icon_recipe): ButtonState = crate::button::look::button_icon,
        checkbox(checkbox_recipe): CheckableState = crate::toggle::look::checkbox,
        /// The checkbox indicator box. Default recipes draw the indicator in the root.
        checkbox_indicator(checkbox_indicator_recipe): CheckableState = crate::toggle::look::empty_toggle_part,
        /// A persistent part inside the checkbox indicator for the checked mark.
        checkbox_check(checkbox_check_recipe): CheckableState = crate::toggle::look::empty_toggle_part,
        /// A persistent part inside the checkbox indicator for the indeterminate mark.
        checkbox_mixed(checkbox_mixed_recipe): CheckableState = crate::toggle::look::empty_toggle_part,
        switch(switch_recipe): CheckableState = crate::toggle::look::switch,
        /// The switch track. Default recipes draw the track and thumb in the root.
        switch_track(switch_track_recipe): CheckableState = crate::toggle::look::empty_toggle_part,
        /// A persistent part inside the track for the moving thumb.
        switch_thumb(switch_thumb_recipe): CheckableState = crate::toggle::look::empty_toggle_part,
        /// A [`toggle_button`](super::toggle) button: the icon and the label in a row.
        toggle(toggle_recipe): ToggleButtonState = crate::toggle_button::look::toggle_button,
        text_input(text_input_recipe): TextInputState = crate::text_input::look::text_input,
        text_area(text_area_recipe): TextInputState = crate::text_input::look::text_area,
    }
    tables {
        stat(stat_recipes): StatRecipes,
        description_list(description_list_recipes): DescriptionListRecipes,
        chat_message(chat_message_recipes): ChatMessageRecipes,
        chat_thread(chat_thread_recipes): ChatThreadRecipes,
        chat_composer(chat_composer_recipes): ChatComposerRecipes,
        typing_indicator(typing_indicator_recipes): TypingIndicatorRecipes,
        code_block(code_block_recipes): CodeBlockRecipes,
        empty_state(empty_state_recipes): EmptyStateRecipes,
        alert(alert_recipes): AlertRecipes,
        card(card_recipes): CardRecipes,
        input_group(input_group_recipes): InputGroupRecipes,
        calendar(calendar_recipes): CalendarRecipes,
        date_input(date_input_recipes): DateInputRecipes,
        date_range_input(date_range_input_recipes): DateRangeInputRecipes,
        time_input(time_input_recipes): TimeInputRecipes,
        date_time_input(date_time_input_recipes): DateTimeInputRecipes,
        combobox(combobox_recipes): ComboboxRecipes,
        toolbar(toolbar_recipes): ToolbarRecipes,
        pagination(pagination_recipes): PaginationRecipes,
        breadcrumb(breadcrumb_recipes): BreadcrumbRecipes,
        stepper(stepper_recipes): StepperRecipes,
        navigation_menu(navigation_menu_recipes): NavigationMenuRecipes,
        toggle_group(toggle_group_recipes): ToggleGroupRecipes,
        button_group(button_group_recipes): ButtonGroupRecipes,
        select(select_recipes): SelectRecipes,
        menu(menu_recipes): MenuRecipes,
        dialog(dialog_recipes): DialogRecipes,
        tooltip(tooltip_recipes): TooltipRecipes,
        hover_card(hover_card_recipes): HoverCardRecipes,
        tabs(tabs_recipes): TabsRecipes,
        radio_group(radio_group_recipes): RadioGroupRecipes,
        slider(slider_recipes): SliderRecipes,
        toast(toast_recipes): ToastRecipes,
        field(field_recipes): FieldRecipes,
        splitter(splitter_recipes): SplitterRecipes,
        list(list_recipes): ListRecipes,
        tree(tree_recipes): TreeRecipes,
        table(table_recipes): TableRecipes,
        item_list(item_list_recipes): ItemListRecipes,
        data_table(data_table_recipes): DataTableRecipes,
        split(split_recipes): SplitRecipes,
        progress(progress_recipes): ProgressRecipes,
        sparkline(sparkline_recipes): SparklineRecipes,
        disclosure(disclosure_recipes): DisclosureRecipes,
        carousel(carousel_recipes): CarouselRecipes,
        dropzone(dropzone_recipes): DropzoneRecipes,
        command_palette(command_palette_recipes): CommandPaletteRecipes,
        tag_input(tag_input_recipes): TagInputRecipes,
        number_input(number_input_recipes): NumberInputRecipes,
    }
}

impl ThemeExtension for UiRecipes {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}

impl UiRecipes {
    pub(super) fn has_checkbox_parts(&self) -> bool {
        let default = Self::default_table();
        !self.checkbox_indicator.same(&default.checkbox_indicator)
            || !self.checkbox_check.same(&default.checkbox_check)
            || !self.checkbox_mixed.same(&default.checkbox_mixed)
    }

    pub(super) fn has_switch_parts(&self) -> bool {
        let default = Self::default_table();
        !self.switch_track.same(&default.switch_track)
            || !self.switch_thumb.same(&default.switch_thumb)
    }
}
