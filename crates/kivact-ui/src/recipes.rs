//! Role: Passive function tables define every standard component's theme recipes.
//!
//! A look has three layers: tokens (the theme's palette, sizes,
//! `focus_ring`, `disabled_opacity` and `icons`, plus a kit's tokens, a `ThemeExtension`),
//! recipes (the style of each part) and slots (the view an item shows).
//! [`Recipe`] is the only shape of a recipe; it is `Send + Sync` so that a
//! theme can be passed from a worker to the UI. A recipe never returns a
//! view: what an item shows belongs to the component's slot (`Tabs::item`,
//! `RadioGroup::item`, `Menu::item`, `Select::option`, each given the item
//! and its state as `Value`s), and without a slot the component draws the
//! item's icon, label, description, check and radio marks and headings.
//!
//! [`UiRecipes`] goes into a theme with `Theme::with` (it is a
//! `ThemeExtension`, one table per type); a theme without one draws the
//! default table. Button, toggle, checkbox, switch, text input and text area have their
//! recipes on it, and select, menu, dialog, tooltip, tabs, radio group,
//! slider, toast, field, splitter, list, data table, split, progress, disclosure,
//! command palette, tag input and number input each
//! have a small
//! table of per-part recipes.
//! The fields are private: the only way to change a recipe is the method of
//! the part's name, which takes the current recipe and returns the new one,
//! and the way to read it is `<part>_recipe()` (a sub-table is
//! `<table>_recipes()`, plural, so it does not collide with the overriding
//! `button` or `tabs`). Components and kits read through those methods.
//! Tables compare with `Arc::ptr_eq`, and a theme change restyles.
//!
//! A recipe writes hover, press, focus and disabled looks as the returned
//! style's state variants (`hover`, `pressed`, `focus`, `focus_visible`,
//! `disabled`) and does not branch on `state.input`; the exception is text
//! decoration, which state styles cannot set (the default `Link` underlines
//! while hovered). Local drawing is the recipe's `Style::paint`, with no
//! child node added only to hold it, and structural parts (an icon box, the
//! busy indicator, a menu mark) are their own nodes with their own recipes.
//! A root's style layers a one-line text setting, then the recipe, then the
//! caller's `.style`, so the caller's modifiers win; drawing runs the
//! caller's `.draw`, the recipe's paint, the children and the caller's
//! `.draw_over` in that order, and a style merge keeps a paint it does not
//! set. The marks of items are icons from the theme's `IconSet` that the
//! component puts in the recipe's box, never drawn by the recipe: `check` for
//! a menu's check item, `dot` for a radio group's item, `menu-radio` for a
//! menu's radio item (the same image as
//! `dot` by default; a theme that marks it with a check gives another),
//! `chevron-right` for a submenu, `chevron-down` and the selected option's
//! `check` for a select, and `x` for the close of a dialog and a toast. A
//! look without a mark makes the recipe's box zero-sized.
use super::values::{
    AxisState, ButtonState, CheckState, DataTableCellState, DataTableCheckState,
    DataTablePageState, DataTableRowState, DialogState, FieldState, ProgressState,
    SelectTriggerState, TextInputState, ToastGroupState, ToggleButtonState, TooltipArrowState,
};
use super::variants::Look;
use super::{look, look_collections, look_controls, look_extra, look_input, look_palette};
use kivact_core::primitive::PartState;
use kivact_core::style::{Style, Theme, ThemeExtension};
use kivact_core::view::Orientation;
use kivact_interaction::{
    DisclosureState, ListItemState, MenuItemState, PopupState, TabListState, TabState,
    TableHeaderState, ToastState,
};
use std::sync::{Arc, OnceLock};

/// A style derived from the theme and the actual state of a component's
/// part with the component's kit-defined variants ([`Look`]): what a
/// recipe table holds for one part.
///
/// A recipe returns only style (layout, paint, local `Style::paint`
/// drawing, and the hover, press, focus and disabled looks as state
/// styles); what a part shows is the component's, or a slot the caller
/// fills (`Tabs::item`, `Menu::item`, `Select::option`). Tables compare
/// their recipes by identity, so installing an unchanged table never
/// restyles.
pub type Recipe<S> = Arc<dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync>;

/// A recipe table: one recipe per part and the tables of the parts'
/// components. A component reads a recipe with the part's `_recipe` method
/// (a table with `_recipes`); a theme changes one with the method of the
/// part's own name, which takes the current recipe and returns the new one,
/// so it wraps or replaces the recipes it changes and keeps the others. The
/// fields are private: that method is the only way to change a table.
macro_rules! recipe_table {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$fmeta:meta])* $field:ident ($read:ident) : $state:ty = $default:path, )*
        }
        $( tables { $( $(#[$tmeta:meta])* $table:ident ($tread:ident) : $tty:ty, )* } )?
    ) => {
        $(#[$meta])*
        #[derive(Clone)]
        pub struct $name {
            $( $field: Recipe<$state>, )*
            $( $( $table: $tty, )* )?
        }

        impl $name {
            fn default_table() -> &'static Self {
                static DEFAULT: OnceLock<$name> = OnceLock::new();
                DEFAULT.get_or_init(|| Self {
                    $( $field: Arc::new(|theme: &Theme, look: &Look<$state>| $default(theme, look)), )*
                    $( $( $table: Default::default(), )* )?
                })
            }

            $(
                $(#[$fmeta])*
                ///
                /// This reads the recipe; the method of the same name without
                /// `_recipe` replaces it.
                pub fn $read(&self) -> &Recipe<$state> {
                    &self.$field
                }

                $(#[$fmeta])*
                ///
                /// This replaces the recipe: `f` gets the current recipe and
                /// returns the new one, which may call it.
                pub fn $field<F>(mut self, f: impl FnOnce(Recipe<$state>) -> F) -> Self
                where
                    F: Fn(&Theme, &Look<$state>) -> Style + Send + Sync + 'static,
                {
                    self.$field = Arc::new(f(self.$field));
                    self
                }
            )*
            $( $(
                $(#[$tmeta])*
                ///
                /// This reads the table; the method of the same name without
                /// `_recipes` replaces it.
                pub fn $tread(&self) -> &$tty {
                    &self.$table
                }

                $(#[$tmeta])*
                ///
                /// This replaces the table: `f` gets the current table and
                /// returns the new one.
                pub fn $table(mut self, f: impl FnOnce($tty) -> $tty) -> Self {
                    self.$table = f(self.$table);
                    self
                }
            )* )?
        }

        impl Default for $name {
            fn default() -> Self {
                Self::default_table().clone()
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                true
                    $( && Arc::ptr_eq(&self.$field, &other.$field) )*
                    $( $( && self.$table == other.$table )* )?
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    $( .field(stringify!($field), &Arc::as_ptr(&self.$field).cast::<()>()) )*
                    $( $( .field(stringify!($table), &self.$table) )* )?
                    .finish()
            }
        }
    };
}

recipe_table! {
    /// The recipes of a [`select`](super::select).
    SelectRecipes {
        /// The trigger, which knows whether it shows the placeholder. Its
        /// [`indicator`](Self::indicator_recipe) holds the theme's `chevron-down`
        /// icon.
        trigger(trigger_recipe): SelectTriggerState = look_extra::select_trigger,
        /// The floating list.
        surface(surface_recipe): PopupState = look_extra::surface,
        /// An option row: its icon, its label and its description.
        option(option_recipe): ListItemState = look_extra::option,
        /// The trailing box of an option, holding the theme's `check` icon
        /// while the option is selected. Every option has the box, so the
        /// labels line up; a custom [`Select::option`](super::Select::option)
        /// row shows no check.
        check(check_recipe): ListItemState = look_extra::option_check,
        /// The box of the trigger's `chevron-down` icon.
        indicator(indicator_recipe): PopupState = look_extra::select_indicator,
        /// The secondary line of an option with a description.
        description(description_recipe): ListItemState = look_extra::option_description,
    }
}

recipe_table! {
    /// The recipes of a [`menu`](super::menu).
    MenuRecipes {
        /// The menu's own trigger button (without `Menu::attach`).
        trigger(trigger_recipe): PopupState = look_extra::menu_trigger,
        /// The top level's floating surface.
        surface(surface_recipe): PopupState = look_extra::surface,
        /// A submenu's floating surface (by default the top level's look).
        sub_surface(sub_surface_recipe): PopupState = look_extra::surface,
        /// The top level's surface of a menu opened by
        /// [`attach_context`](super::Menu::attach_context) (by default the
        /// top level's look).
        context_surface(context_surface_recipe): PopupState = look_extra::surface,
        /// An item row: its marks, icon, label, shortcut and chevron.
        item(item_recipe): MenuItemState = look_extra::menu_item,
        separator(separator_recipe): PartState = look_extra::separator,
        /// A heading: a [`MenuEntry::label`](kivact_interaction::MenuEntry::label)
        /// or the label of a group.
        label(label_recipe): PartState = look_extra::menu_label,
        shortcut(shortcut_recipe): MenuItemState = look_extra::shortcut,
        /// The secondary line of an item with a description.
        description(description_recipe): MenuItemState = look_extra::menu_description,
        /// The leading box of a check or radio item, holding the theme's
        /// `check` or `menu-radio` icon while it is checked. The box gets the
        /// item's actual [`MenuItemState`], as the submenu box does, even for
        /// a checked submenu item.
        check(check_recipe): MenuItemState = look_extra::menu_check,
        /// The trailing box of a submenu item, holding the theme's
        /// `chevron-right` icon.
        submenu(submenu_recipe): MenuItemState = look_extra::menu_submenu,
        /// The empty box before the content of an
        /// [`inset`](kivact_interaction::MenuEntry::inset) item or heading:
        /// by default the width of the check box, so it lines up with the
        /// check and radio items.
        inset(inset_recipe): PartState = look_extra::menu_inset,
    }
}

recipe_table! {
    /// The recipes of a [`dialog`](super::dialog).
    DialogRecipes {
        surface(surface_recipe): DialogState = look_extra::dialog_surface,
        title(title_recipe): DialogState = look_extra::dialog_title,
        description(description_recipe): DialogState = look_extra::dialog_description,
        header(header_recipe): DialogState = look_extra::dialog_header,
        /// The box around the header's [`media`](super::Dialog::media) view.
        media(media_recipe): DialogState = look_extra::dialog_media,
        body(body_recipe): DialogState = look_extra::dialog_body,
        footer(footer_recipe): DialogState = look_extra::dialog_footer,
        scrim(scrim_recipe): DialogState = look_extra::dialog_scrim,
        /// The header's close button, holding the theme's `x` icon.
        close(close_recipe): DialogState = look_extra::dialog_close,
    }
}

recipe_table! {
    /// The recipes of a [`tooltip`](super::tooltip).
    TooltipRecipes {
        surface(surface_recipe): PopupState = look_extra::tooltip,
        /// The arrow toward the trigger: an absolutely positioned box in the
        /// surface, given the side the tooltip shows on and where the
        /// trigger's centre is along that edge. The default draws no arrow.
        arrow(arrow_recipe): TooltipArrowState = look_extra::no_arrow,
    }
}

recipe_table! {
    /// The recipes of [`tabs`](super::tabs).
    TabsRecipes {
        /// The tab list, which knows its orientation.
        list(list_recipe): TabListState = look_extra::tabs_list,
        /// A tab, which knows whether it is selected and the list's orientation.
        tab(tab_recipe): TabState = look_extra::tab,
        panel(panel_recipe): TabState = look_extra::panel,
    }
}

recipe_table! {
    /// The recipes of a [`radio_group`](super::radio_group).
    RadioGroupRecipes {
        /// The group, laid out in its orientation.
        group(group_recipe): Orientation = look_controls::radio_group,
        /// A radio item: its indicator and its label.
        item(item_recipe): CheckState = look_controls::radio_item,
        /// The item's indicator, holding the theme's `dot` icon while
        /// it is checked.
        indicator(indicator_recipe): CheckState = look_controls::radio_indicator,
        /// The dot icon, mounted through selection changes so its paint can transition.
        mark(mark_recipe): CheckState = look_controls::radio_mark,
        /// The secondary line of an option with a description.
        description(description_recipe): CheckState = look_controls::radio_description,
    }
}

recipe_table! {
    /// The recipes of a [`slider`](super::slider) and a
    /// [`range_slider`](super::range_slider).
    SliderRecipes {
        /// The track the pointer moves along; the component places the
        /// range and the thumbs in it.
        track(track_recipe): AxisState = look_controls::slider_track,
        /// The filled part of the track, from its start (or the first
        /// thumb) to the thumb.
        range(range_recipe): AxisState = look_controls::slider_range,
        /// A thumb. Its size is the recipe's; the component places it.
        thumb(thumb_recipe): AxisState = look_controls::slider_thumb,
    }
}

recipe_table! {
    /// The recipes of a [`toast_region`](super::toast_region).
    ToastRecipes {
        /// The region, placed at the window's bottom right.
        region(region_recipe): PartState = look_controls::toast_region,
        /// The column of one live region's toasts (polite, then assertive).
        group(group_recipe): ToastGroupState = look_controls::toast_group,
        /// One toast: its title, its body, its action and its close button.
        toast(toast_recipe): ToastState = look_controls::toast,
        title(title_recipe): ToastState = look_controls::toast_title,
        body(body_recipe): ToastState = look_controls::toast_body,
        /// The icon of the toast's tone, which the theme's `IconSet` names
        /// `toast-info`, `toast-success`, `toast-warning`, `toast-danger`
        /// and `toast-accent` (a neutral toast has none, and a toast with its
        /// own [`Toast::icon`](kivact_interaction::Toast::icon) draws that
        /// view instead).
        icon(icon_recipe): ToastState = look_controls::toast_icon,
        /// The close button, holding the theme's `x` icon.
        close(close_recipe): ToastState = look_controls::toast_close,
    }
}

recipe_table! {
    /// The recipes of a [`field`](super::field).
    FieldRecipes {
        /// The column of label, control, hint and error.
        root(root_recipe): FieldState = look_extra::field,
        /// The visible label, which knows whether the field has an error.
        label(label_recipe): FieldState = look_extra::field_label,
        /// The hint under the control.
        hint(hint_recipe): FieldState = look_extra::field_hint,
        /// The error message under the hint.
        error(error_recipe): FieldState = look_extra::field_error,
    }
}

recipe_table! {
    /// The recipes of a [`splitter`](super::splitter).
    SplitterRecipes {
        /// The handle between the panes.
        handle(handle_recipe): AxisState = look_controls::splitter_handle,
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
    /// The recipes of a [`split`](super::split), besides its handles'
    /// [`SplitterRecipes::handle`].
    SplitRecipes {
        /// The grip on a handle, shown while [`Split::grip`](super::Split::grip)
        /// is true; its state is the handle's.
        grip(grip_recipe): AxisState = look_controls::split_grip,
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
        track(track_recipe): ProgressState = look_extra::progress_track,
        /// The fill, the track's one child: the whole track, painted or
        /// sized to the value.
        fill(fill_recipe): ProgressState = look_extra::progress_fill,
    }
}

recipe_table! {
    /// The recipes of a [`disclosure`](super::disclosure); all of them get
    /// the trigger's or the content's [`DisclosureState`].
    DisclosureRecipes {
        /// The column of the trigger over the content.
        root(root_recipe): DisclosureState = look_extra::disclosure_root,
        /// The trigger: the title and the indicator in a row.
        trigger(trigger_recipe): DisclosureState = look_extra::disclosure_trigger,
        /// The box of the trigger's `chevron-down` icon, which a look turns
        /// while the content is open.
        indicator(indicator_recipe): DisclosureState = look_extra::disclosure_indicator,
        /// The content under the trigger.
        content(content_recipe): DisclosureState = look_extra::disclosure_content,
    }
}

recipe_table! {
    /// The recipes of a [`command_palette`](super::command_palette).
    CommandPaletteRecipes {
        /// The layer that covers the window behind the modal.
        scrim(scrim_recipe): PartState = look_palette::scrim,
        /// The modal's panel, which holds the search, the list and the footer.
        panel(panel_recipe): PopupState = look_palette::panel,
        /// The row of the magnifier and the search field.
        search(search_recipe): PartState = look_palette::search,
        /// The magnifier before the field, drawn while the theme's `IconSet`
        /// has a `search` image. A look without one draws none, and a look
        /// that does not want it gives the box no size.
        search_icon(search_icon_recipe): PartState = look_palette::search_icon,
        /// The search field (a native editing leaf: its style has no paint).
        field(field_recipe): PartState = look_palette::field,
        /// The scrolling box of the groups and rows.
        list(list_recipe): PartState = look_palette::list,
        /// The heading of a group.
        group(group_recipe): PartState = look_palette::group,
        /// A command's row; `active` marks the row Enter runs, and the row
        /// is `selected` while it is.
        row(row_recipe): ListItemState = look_palette::row,
        /// The icon at the start of a row.
        icon(icon_recipe): ListItemState = look_palette::icon,
        /// The hint at the end of a row.
        hint(hint_recipe): ListItemState = look_palette::hint,
        /// The text shown when no command matches.
        empty(empty_recipe): PartState = look_palette::empty,
        /// The line under the list, drawn only when the caller gives a
        /// [`footer`](super::CommandPalette::footer).
        footer(footer_recipe): PartState = look_palette::footer,
    }
}

recipe_table! {
    /// The recipes of a [`tag_input`](super::tag_input).
    TagInputRecipes {
        /// The box of the tags and the input; its `focused` is true while the
        /// focus is in the input.
        root(root_recipe): PartState = look_input::tag_root,
        /// One tag: its text and its remove button.
        tag(tag_recipe): PartState = look_input::tag,
        /// A tag's remove button, over the button recipes.
        remove(remove_recipe): PartState = look_input::tag_remove,
        /// The input, over the text input recipe.
        input(input_recipe): PartState = look_input::tag_input,
    }
}

recipe_table! {
    /// The recipes of a [`number_input`](super::number_input).
    NumberInputRecipes {
        /// The box of the decrease button, the input and the increase button.
        root(root_recipe): PartState = look_input::number_root,
        /// The input, over the text input recipe.
        input(input_recipe): PartState = look_input::number_input,
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
    /// let recipes = theme.get::<UiRecipes>();
    /// let square = UiRecipes::clone(&recipes)
    ///     .button(|prev| move |theme, state| prev(theme, state).radius(0))
    ///     .menu(|menu| menu.item(|prev| move |theme, state| prev(theme, state).padding_x(16)));
    /// let theme = theme.with(square);
    /// # let _ = theme;
    /// ```
    UiRecipes {
        button(button_recipe): ButtonState = look::button,
        /// The row of a button's icons, label and children.
        button_content(button_content_recipe): ButtonState = look::button_content,
        /// The busy indicator while `loading`; the button turns it about
        /// its centre.
        button_busy(button_busy_recipe): ButtonState = look::button_busy,
        /// The box around a button's leading and trailing icon: where a kit
        /// sizes the icon for the button's size.
        button_icon(button_icon_recipe): ButtonState = look::button_icon,
        checkbox(checkbox_recipe): CheckState = look::checkbox,
        /// The checkbox indicator box. Default recipes draw the indicator in the root.
        checkbox_indicator(checkbox_indicator_recipe): CheckState = look_controls::empty_toggle_part,
        /// A persistent part inside the checkbox indicator for the checked mark.
        checkbox_check(checkbox_check_recipe): CheckState = look_controls::empty_toggle_part,
        /// A persistent part inside the checkbox indicator for the indeterminate mark.
        checkbox_mixed(checkbox_mixed_recipe): CheckState = look_controls::empty_toggle_part,
        switch(switch_recipe): CheckState = look::switch,
        /// The switch track. Default recipes draw the track and thumb in the root.
        switch_track(switch_track_recipe): CheckState = look_controls::empty_toggle_part,
        /// A persistent part inside the track for the moving thumb.
        switch_thumb(switch_thumb_recipe): CheckState = look_controls::empty_toggle_part,
        /// A [`toggle`](super::toggle) button: the icon and the label in a row.
        toggle(toggle_recipe): ToggleButtonState = look::toggle_button,
        text_input(text_input_recipe): TextInputState = look::text_input,
        text_area(text_area_recipe): TextInputState = look::text_area,
    }
    tables {
        select(select_recipes): SelectRecipes,
        menu(menu_recipes): MenuRecipes,
        dialog(dialog_recipes): DialogRecipes,
        tooltip(tooltip_recipes): TooltipRecipes,
        tabs(tabs_recipes): TabsRecipes,
        radio_group(radio_group_recipes): RadioGroupRecipes,
        slider(slider_recipes): SliderRecipes,
        toast(toast_recipes): ToastRecipes,
        field(field_recipes): FieldRecipes,
        splitter(splitter_recipes): SplitterRecipes,
        list(list_recipes): ListRecipes,
        data_table(data_table_recipes): DataTableRecipes,
        split(split_recipes): SplitRecipes,
        progress(progress_recipes): ProgressRecipes,
        disclosure(disclosure_recipes): DisclosureRecipes,
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
        !Arc::ptr_eq(&self.checkbox_indicator, &default.checkbox_indicator)
            || !Arc::ptr_eq(&self.checkbox_check, &default.checkbox_check)
            || !Arc::ptr_eq(&self.checkbox_mixed, &default.checkbox_mixed)
    }

    pub(super) fn has_switch_parts(&self) -> bool {
        let default = Self::default_table();
        !Arc::ptr_eq(&self.switch_track, &default.switch_track)
            || !Arc::ptr_eq(&self.switch_thumb, &default.switch_thumb)
    }
}

impl RadioGroupRecipes {
    pub(super) fn has_custom_mark(&self) -> bool {
        !Arc::ptr_eq(&self.mark, &Self::default_table().mark)
    }
}
