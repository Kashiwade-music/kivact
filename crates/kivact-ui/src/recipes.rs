use crate::combobox::look::ComboboxRecipes;
use crate::command_palette::look::CommandPaletteRecipes;
use crate::dialog::look::DialogRecipes;
use crate::disclosure::look::DisclosureRecipes;
use crate::hover_card::look::HoverCardRecipes;
use crate::menu::look::MenuRecipes;
use crate::navigation_menu::look::NavigationMenuRecipes;
use crate::select::look::SelectRecipes;
use crate::toast::look::ToastRecipes;
use crate::tooltip::look::TooltipRecipes;

use super::look_feedback;
use super::values::{
    DataTableCellState, DataTableCheckState, DataTablePageState, DataTableRowState, ProgressState,
};
use super::{look_collections, look_controls, look_extra};
use crate::button::look::ButtonState;
use crate::button_group::look::ButtonGroupRecipes;
use crate::component_values::{AlertState, CardState, EmptyStateState, HeadingState, StatState};
use crate::dropzone::look::DropzoneRecipes;
use crate::field::look::FieldRecipes;
use crate::input_group::look::InputGroupRecipes;
use crate::number_input::look::NumberInputRecipes;
use crate::recipe::recipe_table;
use crate::slider::look::SliderRecipes;
use crate::split::look::SplitRecipes;
use crate::splitter::look::SplitterRecipes;
use crate::table_values::DataTableGroupState;
use crate::tag_input::look::TagInputRecipes;
use crate::text_input::look::TextInputState;
use crate::toggle::look::CheckableState;
use crate::toggle_button::look::ToggleButtonState;
use crate::toggle_group::look::ToggleGroupRecipes;
use crate::toolbar::look::ToolbarRecipes;
use crate::{look_files, look_metrics, look_static_collections};
use kivact_core::custom::PartState;
use kivact_core::style::{Theme, ThemeExtension};
use kivact_core::view::Orientation;
use kivact_interaction::{ListItemState, TabListState, TabState, TableHeaderState};

recipe_table! {
    /// Calendar panels and date cells.
    CalendarRecipes {
        root(root_recipe): PartState = super::look_calendar::root,
        month(month_recipe): PartState = super::look_calendar::month,
        caption(caption_recipe): PartState = super::look_calendar::month,
        events(events_recipe): PartState = super::look_calendar::events,
        event(event_recipe): crate::calendar_tokens::CalendarEventVisual = super::look_calendar::event,
        day(day_recipe): crate::calendar_values::CalendarDayVisual = super::look_calendar::day,
        range(range_recipe): kivact_interaction::CalendarDayState = super::look_calendar::range,
    }
}
recipe_table! {
    /// Date and time field layouts and touch wheel surfaces.
    DateInputRecipes {
        root(root_recipe): PartState = super::look_calendar::input,
        wheel(wheel_recipe): PartState = super::look_calendar::wheel,
    }
}
recipe_table! {
    /// The parts of a date-range field.
    DateRangeInputRecipes { root(root_recipe): PartState = super::look_calendar::input, }
}
recipe_table! {
    /// The parts of a wall-clock field.
    TimeInputRecipes {
        root(root_recipe): PartState = super::look_calendar::input,
        wheel(wheel_recipe): PartState = super::look_calendar::wheel,
    }
}
recipe_table! {
    /// The parts of a date-and-time field.
    DateTimeInputRecipes { root(root_recipe): PartState = super::look_calendar::input, }
}

recipe_table! {
    /// The recipes of [`tabs`](super::tabs).
    TabsRecipes {
        /// The tab list, which knows its orientation.
        list(list_recipe): TabListState = look_extra::tabs_list,
        /// A tab, which knows whether it is selected and the list's orientation.
        tab(tab_recipe): TabState = look_extra::tab,
        /// An optional, persistent decorative layer per tab. It receives
        /// both selected and unselected states so its paint can transition.
        indicator(indicator_recipe): TabState = look_extra::no_tab_indicator,
        panel(panel_recipe): TabState = look_extra::panel,
    }
}

recipe_table! {
    /// The recipes of a [`radio_group`](super::radio_group).
    RadioGroupRecipes {
        /// The group, laid out in its orientation.
        group(group_recipe): Orientation = super::look_radio::group,
        /// A radio item: its indicator and its label.
        item(item_recipe): CheckableState = super::look_radio::item,
        /// The item's indicator, holding the theme's `dot` icon while
        /// it is checked.
        indicator(indicator_recipe): CheckableState = super::look_radio::indicator,
        /// The dot icon, mounted through selection changes so its paint can transition.
        mark(mark_recipe): CheckableState = look_controls::radio_mark,
        /// The secondary line of an option with a description.
        description(description_recipe): CheckableState = look_controls::radio_description,
    }
}

recipe_table! {
    /// The recipes of a [`list`](super::list).
    ListRecipes {
        /// The column of items.
        root(root_recipe): PartState = look_collections::list,
        /// An item row: its icon, its label and its description.
        item(item_recipe): ListItemState = look_collections::list_item,
        /// The secondary line of an item with a description.
        description(description_recipe): ListItemState = look_collections::list_description,
    }
}

recipe_table! {
    /// The recipes of a [`tree`](super::tree).
    TreeRecipes {
        /// The column containing the visible rows.
        root(root_recipe): PartState = look_collections::list,
        /// A row with the expansion indicator and its content.
        item(item_recipe): crate::tree_values::TreeRowRecipeState = look_collections::tree_item,
        /// The branch indicator or matching leaf spacer.
        marker(marker_recipe): crate::tree_values::TreeRowRecipeState = look_collections::tree_marker,
    }
}

recipe_table! {
    /// The recipes of a [`data_table`](super::data_table). What a header, a
    /// cell or the toolbar shows is the table's own or a slot the caller
    /// fills ([`DataTable::toolbar`](super::DataTable::toolbar),
    /// [`footer`](super::DataTable::footer),
    /// [`cell_controls`](super::DataTable::cell_controls)).
    DataTableRecipes {
        /// The frame around the toolbar, the grid and the footer.
        frame(frame_recipe): PartState = look_collections::frame,
        /// The row above the grid, holding the filter field or the
        /// toolbar slot.
        toolbar(toolbar_recipe): PartState = look_collections::toolbar,
        /// The row under the grid, holding the summary and the pager or the
        /// footer slot.
        footer(footer_recipe): PartState = look_collections::footer,
        /// A column header; it knows whether its column sorts and in which
        /// direction. The column's width is the table's.
        header(header_recipe): TableHeaderState = look_collections::header,
        /// A body row; it knows whether it is selected and the caller's row
        /// height and accent.
        row(row_recipe): DataTableRowState = look_collections::row,
        /// A group header spanning the visible columns.
        group_header(group_header_recipe): DataTableGroupState = look_collections::group_header,
        /// A body cell.
        cell(cell_recipe): DataTableCellState = look_collections::cell,
        /// The box around a check box, in a header or a row.
        check(check_recipe): DataTableCheckState = look_collections::check,
        /// The sort mark of a header, drawn by the recipe in its own box. A
        /// look that gives the box no size shows no mark.
        sort(sort_recipe): TableHeaderState = look_collections::sort,
        /// The pager under the grid: Previous, Next and the page shown.
        page(page_recipe): DataTablePageState = look_collections::page,
    }
}

recipe_table! {
    /// The recipes of a [`progress`](super::progress) bar.
    ProgressRecipes {
        /// The bar itself: the track the fill moves in, which clips it.
        track(track_recipe): ProgressState = look_metrics::progress_track,
        /// The fill, the track's one child: the whole track, painted or
        /// sized to the value.
        fill(fill_recipe): ProgressState = look_metrics::progress_fill,
        /// Percentage text in a ring.
        label(label_recipe): ProgressState = look_metrics::progress_label,
    }
}

recipe_table! {
    /// The recipes of a [`sparkline`](super::sparkline).
    SparklineRecipes {
        /// The chart's box: its size. The theme's `UiTokens::sparkline` draws the line and the area in it.
        root(root_recipe): () = look_metrics::sparkline_root,
    }
}

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
        heading(heading_recipe): HeadingState = look_metrics::heading,
        avatar_status_content(avatar_status_content_recipe): crate::indicator_values::AvatarState = super::look_indicators::avatar_status_content,
        avatar_status(avatar_status_recipe): crate::indicator_values::AvatarState = super::look_indicators::avatar_status,
        avatar_group(avatar_group_recipe): () = super::look_indicators::avatar_group,
        avatar_group_member(avatar_group_member_recipe): usize = super::look_indicators::avatar_group_member,
        /// Presentation of the hidden avatar count, including its actual count.
        avatar_group_overflow(avatar_group_overflow_recipe): usize = super::look_indicators::avatar_group_overflow,
        avatar(avatar_recipe): crate::indicator_values::AvatarState = super::look_indicators::avatar,
        avatar_image(avatar_image_recipe): crate::indicator_values::AvatarState = super::look_indicators::avatar_image,
        kbd(kbd_recipe): () = super::look_indicators::kbd,
        kbd_key(kbd_key_recipe): crate::kbd_values::KbdKeyState = super::look_indicators::kbd_key,
        badge(badge_recipe): crate::indicator_values::BadgeState = super::look_indicators::badge,
        badge_label(badge_label_recipe): crate::indicator_values::BadgeState = super::look_indicators::badge_label,
        spinner(spinner_recipe): crate::indicator_values::SpinnerState = super::look_indicators::spinner,
        skeleton(skeleton_recipe): crate::indicator_values::SkeletonState = super::look_indicators::skeleton,
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

impl RadioGroupRecipes {
    pub(super) fn has_custom_mark(&self) -> bool {
        !self.mark.same(&Self::default_table().mark)
    }
}

impl TabsRecipes {
    pub(super) fn has_custom_indicator(&self) -> bool {
        !self.indicator.same(&Self::default_table().indicator)
    }
}

recipe_table! {
    /// Styles for a carousel's persistent parts.
    CarouselRecipes {
        root(root_recipe): crate::carousel_values::CarouselSurfaceState = look_files::carousel_root,
        viewport(viewport_recipe): crate::carousel_values::CarouselSurfaceState = look_files::carousel_viewport,
        track(track_recipe): crate::carousel_values::CarouselSurfaceState = look_files::carousel_track,
        slide(slide_recipe): crate::carousel_values::CarouselSurfaceState = look_files::carousel_slide,
        previous(previous_recipe): crate::carousel_values::CarouselSurfaceState = look_files::carousel_control,
        next(next_recipe): crate::carousel_values::CarouselSurfaceState = look_files::carousel_control,
    }
}

recipe_table! {
    /// Recipes for an empty region's message and actions.
    EmptyStateRecipes {
        root(root_recipe): EmptyStateState = look_feedback::empty_root,
        title(title_recipe): EmptyStateState = look_feedback::empty_title,
        description(description_recipe): EmptyStateState = look_feedback::empty_description,
        icon(icon_recipe): EmptyStateState = look_feedback::empty_icon,
        actions(actions_recipe): EmptyStateState = look_feedback::empty_actions,
    }
}
recipe_table! {
    /// Recipes for a persistent alert and optional details.
    AlertRecipes {
        root(root_recipe): AlertState = look_feedback::alert_root,
        header(header_recipe): AlertState = look_feedback::alert_wrapper,
        message(message_recipe): AlertState = look_feedback::alert_wrapper,
        controls(controls_recipe): AlertState = look_feedback::alert_wrapper,
        title(title_recipe): AlertState = look_feedback::alert_title,
        description(description_recipe): AlertState = look_feedback::alert_description,
        icon(icon_recipe): AlertState = look_feedback::alert_icon,
        actions(actions_recipe): AlertState = look_feedback::alert_actions,
        content(content_recipe): AlertState = look_feedback::alert_content,
        dismiss(dismiss_recipe): AlertState = look_feedback::alert_control,
        toggle(toggle_recipe): AlertState = look_feedback::alert_control,
    }
}
recipe_table! {
    /// Recipes for each region of a card.
    CardRecipes {
        root(root_recipe): CardState = look_feedback::card_root,
        header(header_recipe): CardState = look_feedback::card_region,
        media(media_recipe): CardState = look_feedback::card_media,
        content(content_recipe): CardState = look_feedback::card_region,
        footer(footer_recipe): CardState = look_feedback::card_region,
        overlay(overlay_recipe): CardState = look_feedback::card_overlay,
    }
}
recipe_table! {
    /// Static table part recipes.
    TableRecipes {
        root(root_recipe): PartState = look_static_collections::table,
        header(header_recipe): PartState = look_static_collections::group,
        body(body_recipe): PartState = look_static_collections::group,
        footer(footer_recipe): PartState = look_static_collections::footer,
        row(row_recipe): PartState = look_static_collections::row,
        head(head_recipe): PartState = look_static_collections::head,
        cell(cell_recipe): PartState = look_static_collections::cell,
    }
}
recipe_table! {
    /// Static list and item-row part recipes.
    ItemListRecipes {
        list_header(list_header_recipe): PartState = look_static_collections::list_header,
        marker(marker_recipe): crate::collection_values::ItemRowState = look_static_collections::marker,
        root(root_recipe): PartState = look_static_collections::list,
        item(item_recipe): crate::collection_values::ItemRowState = look_static_collections::item,
        body(body_recipe): crate::collection_values::ItemRowState = look_static_collections::item_body,
        content(content_recipe): crate::collection_values::ItemRowState = look_static_collections::content,
        title(title_recipe): crate::collection_values::ItemRowState = look_static_collections::title,
        description(description_recipe): crate::collection_values::ItemRowState = look_static_collections::description,
        media(media_recipe): crate::collection_values::ItemRowState = look_static_collections::media,
        actions(actions_recipe): crate::collection_values::ItemRowState = look_static_collections::actions,
        header(header_recipe): crate::collection_values::ItemRowState = look_static_collections::band,
        footer(footer_recipe): crate::collection_values::ItemRowState = look_static_collections::band,
    }
}
recipe_table! {
    /// Page control and gap recipes.
    PaginationRecipes {
        root(root_recipe): () = crate::look_navigation::root,
        item(item_recipe): crate::navigation_values::PaginationItemState = crate::look_navigation::pagination_item,
        ellipsis(ellipsis_recipe): () = crate::look_navigation::part,
        dots(dots_recipe): () = crate::look_navigation::part,
        controls(controls_recipe): crate::navigation_values::PaginationSummary = crate::look_navigation::pagination_wrapper,
        sizes(sizes_recipe): crate::navigation_values::PaginationSummary = crate::look_navigation::pagination_wrapper,
        input(input_recipe): crate::navigation_values::PaginationSummary = crate::look_navigation::pagination_wrapper,
        summary(summary_recipe): crate::navigation_values::PaginationSummary = crate::look_navigation::part,
    }
}
recipe_table! {
    /// Breadcrumb landmark, item and decorative separator recipes.
    BreadcrumbRecipes {
        root(root_recipe): () = crate::look_navigation::breadcrumb_root,
        list(list_recipe): () = crate::look_navigation::breadcrumb_list,
        item(item_recipe): crate::BreadcrumbItemState = crate::look_navigation::breadcrumb_item,
        separator(separator_recipe): () = crate::look_navigation::part,
    }
}
recipe_table! {
    /// Progress list and individual step recipes.
    StepperRecipes {
        root(root_recipe): crate::navigation_values::StepperState = crate::look_navigation::stepper_root,
        previous(previous_recipe): crate::navigation_values::StepperState = crate::look_navigation::part,
        next(next_recipe): crate::navigation_values::StepperState = crate::look_navigation::part,
        item(item_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::stepper_item,
        layout(layout_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::stepper_layout,
        track(track_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::stepper_layout,
        leading(leading_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::stepper_decoration,
        trailing(trailing_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::stepper_decoration,
        progress(progress_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::stepper_decoration,
        body(body_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::stepper_body,
        indicator(indicator_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::part,
        label(label_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::part,
        description(description_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::part,
        content(content_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::part,
        end_content(end_content_recipe): crate::navigation_values::StepperItemState = crate::look_navigation::part,
    }
}

recipe_table! {
    /// Parts of a statistic and its optional application-owned chart.
    StatRecipes {
        root(root_recipe): StatState = look_metrics::stat_root,
        label(label_recipe): StatState = look_metrics::stat_label,
        value(value_recipe): StatState = look_metrics::stat_value,
        delta(delta_recipe): StatState = look_metrics::stat_delta,
        icon(icon_recipe): StatState = look_metrics::stat_slot,
        chart(chart_recipe): StatState = look_metrics::stat_slot,
    }
}
recipe_table! {
    /// The list, pair, visible term and labelled detail of a description list.
    DescriptionListRecipes {
        root(root_recipe): () = look_metrics::description_root,
        item(item_recipe): () = look_metrics::description_item,
        term(term_recipe): () = look_metrics::description_term,
        detail(detail_recipe): () = look_metrics::description_detail,
    }
}
recipe_table! {
    /// Styles for the persistent parts of ChatMessage.
    ChatMessageRecipes {
        root(root_recipe): crate::ChatMessageState = crate::look_chat::message_root,
        body(body_recipe): crate::ChatMessageState = crate::look_chat::message_body,
        author(author_recipe): crate::ChatMessageState = crate::look_chat::author,
        time(time_recipe): crate::ChatMessageState = crate::look_chat::muted,
        status(status_recipe): crate::ChatMessageState = crate::look_chat::muted,
        content(content_recipe): crate::ChatMessageState = crate::look_chat::slot,
        avatar(avatar_recipe): crate::ChatMessageState = crate::look_chat::slot,
        header(header_recipe): crate::ChatMessageState = crate::look_chat::slot,
        footer(footer_recipe): crate::ChatMessageState = crate::look_chat::slot,
        metadata(metadata_recipe): crate::ChatMessageState = crate::look_chat::slot,
        media(media_recipe): crate::ChatMessageState = crate::look_chat::slot,
        icon(icon_recipe): crate::ChatMessageState = crate::look_chat::slot,
        actions(actions_recipe): crate::ChatMessageState = crate::look_chat::slot,
        reactions(reactions_recipe): crate::ChatMessageState = crate::look_chat::reactions,
    }
}

recipe_table! {
    /// Styles for the persistent parts of ChatThread.
    ChatThreadRecipes {
        root(root_recipe): crate::ChatThreadState = crate::look_chat::thread_root,
        viewport(viewport_recipe): crate::ChatThreadState = crate::look_chat::viewport,
        content(content_recipe): crate::ChatThreadState = crate::look_chat::column,
        scroll_button(scroll_button_recipe): crate::ChatThreadState = crate::look_chat::jump,
        composer(composer_recipe): crate::ChatThreadState = crate::look_chat::slot,
    }
}

recipe_table! {
    /// Styles for the persistent parts of ChatComposer.
    ChatComposerRecipes {
        root(root_recipe): crate::ChatComposerState = crate::look_chat::column,
        surface(surface_recipe): crate::ChatComposerState = crate::look_chat::composer_surface,
        input(input_recipe): crate::ChatComposerState = crate::look_chat::input,
        send(send_recipe): crate::ChatComposerState = crate::look_chat::slot,
        header(header_recipe): crate::ChatComposerState = crate::look_chat::row,
        footer(footer_recipe): crate::ChatComposerState = crate::look_chat::row,
        status(status_recipe): crate::ChatComposerState = crate::look_chat::composer_status,
        drawer(drawer_recipe): crate::ChatComposerState = crate::look_chat::slot,
        header_actions(header_actions_recipe): crate::ChatComposerState = crate::look_chat::slot,
        header_context(header_context_recipe): crate::ChatComposerState = crate::look_chat::slot,
        footer_actions(footer_actions_recipe): crate::ChatComposerState = crate::look_chat::slot,
        send_actions(send_actions_recipe): crate::ChatComposerState = crate::look_chat::slot,
    }
}

recipe_table! {
    /// Styles for the persistent parts of TypingIndicator.
    TypingIndicatorRecipes {
        root(root_recipe): crate::TypingIndicatorState = crate::look_chat::row,
        avatar(avatar_recipe): crate::TypingIndicatorState = crate::look_chat::slot,
        dots(dots_recipe): crate::TypingIndicatorState = crate::look_chat::typing_dots,
    }
}

recipe_table! {
    /// Styles for the persistent parts of CodeBlock.
    CodeBlockRecipes {
        root(root_recipe): crate::code_values::CodeBlockState = crate::look_chat::code_root,
        header(header_recipe): crate::code_values::CodeBlockState = crate::look_chat::code_header,
        title(title_recipe): crate::code_values::CodeBlockState = crate::look_chat::slot,
        collapse(collapse_recipe): crate::code_values::CodeBlockState = crate::look_chat::slot,
        language(language_recipe): crate::code_values::CodeBlockState = crate::look_chat::muted,
        copy(copy_recipe): crate::code_values::CodeBlockState = crate::look_chat::slot,
        end_content(end_content_recipe): crate::code_values::CodeBlockState = crate::look_chat::slot,
        body(body_recipe): crate::code_values::CodeBlockState = crate::look_chat::code_body,
        viewport(viewport_recipe): crate::code_values::CodeBlockState = crate::look_chat::code_viewport,
        code(code_recipe): crate::code_values::CodeBlockState = crate::look_chat::code_text,
        gutter(gutter_recipe): crate::code_values::CodeBlockState = crate::look_chat::code_gutter,
        footer(footer_recipe): crate::code_values::CodeBlockState = crate::look_chat::row,
    }
}
