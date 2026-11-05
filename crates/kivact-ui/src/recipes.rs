//! Role: Passive function tables define every standard component's theme recipes.
//!
//! A look has three layers: tokens (the theme's palette, sizes,
//! `focus_ring`, `disabled_opacity` and `icons`, plus a kit's tokens, a `ThemeExtension`),
//! recipes (the style of each part) and slots (the view an item shows).
//! [`Recipe`] is the only shape of a recipe; it is `Send + Sync` so that a
//! theme can be passed from a worker to the UI. A recipe never returns a
//! view: what an item shows belongs to the component's slot (`Tabs::item`,
//! `RadioGroup::item`, `Menu::item`, `Select::item`, each given the item
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
use super::DataTableGroupState;
use super::values::{
    AxisState, ButtonState, CheckableState, DataTableCellState, DataTableCheckState,
    DataTablePageState, DataTableRowState, DialogState, FieldState, ProgressState,
    SelectTriggerState, TextInputState, ToastGroupState, ToggleButtonState, TooltipArrowState,
};
use super::variants::Look;
use super::{
    AlertState, CardState, EmptyStateState, InputGroupMemberState, InputGroupState, look_feedback,
};
use super::{look, look_collections, look_controls, look_extra, look_input, look_palette};
use crate::{look_files, look_static_collections};
use kivact_core::custom::PartState;
use kivact_core::style::{Style, Theme, ThemeExtension};
use kivact_core::view::Orientation;
use kivact_interaction::{CarouselState, DropzoneState};
use kivact_interaction::{
    DisclosureState, ListItemState, MenuItemState, PopupState, TabListState, TabState,
    TableHeaderState, ToastState, TreeItemState,
};
use std::sync::{Arc, OnceLock};

/// A style derived from the theme and the actual state of a component's
/// part with the component's kit-defined variants ([`Look`]): what a
/// recipe table holds for one part.
///
/// A recipe returns only style (layout, paint, local `Style::paint`
/// drawing, and the hover, press, focus and disabled looks as state
/// styles); what a part shows is the component's, or a slot the caller
/// fills (`Tabs::item`, `Menu::item`, `Select::item`). Tables compare
/// their recipes by identity, so installing an unchanged table never
/// restyles.
pub struct Recipe<S>(Arc<RecipeFn<S>>);

type RecipeFn<S> = dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync;

impl<S> Clone for Recipe<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<S, F> From<F> for Recipe<S>
where
    F: Fn(&Theme, &Look<S>) -> Style + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}
impl<S> std::ops::Deref for Recipe<S> {
    type Target = dyn Fn(&Theme, &Look<S>) -> Style + Send + Sync;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
impl<S> Recipe<S> {
    fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

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
                    $( $field: Recipe::from(|theme: &Theme, look: &Look<$state>| $default(theme, look)), )*
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
                    self.$field = Recipe::from(f(self.$field));
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
                    $( && self.$field.same(&other.$field) )*
                    $( $( && self.$table == other.$table )* )?
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    $( .field(stringify!($field), &Arc::as_ptr(&self.$field.0).cast::<()>()) )*
                    $( $( .field(stringify!($table), &self.$table) )* )?
                    .finish()
            }
        }
    };
}

recipe_table! {
    /// Calendar panels and date cells.
    CalendarRecipes {
        root(root_recipe): PartState = super::look_calendar::root,
        month(month_recipe): PartState = super::look_calendar::month,
        day(day_recipe): super::CalendarDayVisual = super::look_calendar::day,
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
    /// Recipes of the standard combobox family.
    ComboboxRecipes {
        frame(frame_recipe): PartState = super::combobox::look::frame,
        input(input_recipe): PartState = super::combobox::look::input,
        surface(surface_recipe): PartState = super::combobox::look::surface,
        option(option_recipe): ListItemState = super::combobox::look::option,
        chip(chip_recipe): PartState = super::combobox::look::chip,
        remove(remove_recipe): PartState = super::combobox::look::remove,
        empty(empty_recipe): PartState = super::combobox::look::empty,
    }
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
        /// labels line up; a custom [`Select::item`](super::Select::item)
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
        drag_handle(drag_handle_recipe): DialogState = look_extra::drawer_handle,
    }
}

recipe_table! {
    /// Theme surface of a hover preview.
    HoverCardRecipes {
        surface(surface_recipe): crate::HoverCardSurfaceState = look_extra::hover_card,
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
        /// The size-presence shell around one card, including its spacing.
        row(row_recipe): PartState = look_controls::toast_row,
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
    /// The recipes of a [`tree`](super::tree).
    TreeRecipes {
        /// The column containing the visible rows.
        root(root_recipe): PartState = look_collections::list,
        /// A row with the expansion indicator and its content.
        item(item_recipe): TreeItemState = look_collections::tree_item,
        /// The branch indicator or matching leaf spacer.
        marker(marker_recipe): TreeItemState = look_collections::tree_marker,
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
    /// let recipes = theme.ext::<UiRecipes>();
    /// let square = recipes.to_owned()
    ///     .button(|prev| move |theme, state| prev(theme, state).radius(0))
    ///     .menu(|menu| menu.item(|prev| move |theme, state| prev(theme, state).padding_x(16)));
    /// let theme = theme.with(square);
    /// # let _ = theme;
    /// ```
    UiRecipes {
        avatar_status(avatar_status_recipe): crate::AvatarState = super::look_indicators::avatar_status,
        avatar_group(avatar_group_recipe): () = super::look_indicators::avatar_group,
        avatar(avatar_recipe): crate::AvatarState = super::look_indicators::avatar,
        kbd(kbd_recipe): () = super::look_indicators::kbd,
        badge(badge_recipe): crate::BadgeState = super::look_indicators::badge,
        spinner(spinner_recipe): crate::SpinnerState = super::look_indicators::spinner,
        skeleton(skeleton_recipe): crate::SkeletonState = super::look_indicators::skeleton,
        button(button_recipe): ButtonState = look::button,
        /// The row of a button's icons, label and children.
        button_content(button_content_recipe): ButtonState = look::button_content,
        /// The busy indicator while `busy`; the button turns it about
        /// its centre.
        button_busy(button_busy_recipe): ButtonState = look::button_busy,
        /// The box around a button's leading and trailing icon: where a kit
        /// sizes the icon for the button's size.
        button_icon(button_icon_recipe): ButtonState = look::button_icon,
        checkbox(checkbox_recipe): CheckableState = look::checkbox,
        /// The checkbox indicator box. Default recipes draw the indicator in the root.
        checkbox_indicator(checkbox_indicator_recipe): CheckableState = look_controls::empty_toggle_part,
        /// A persistent part inside the checkbox indicator for the checked mark.
        checkbox_check(checkbox_check_recipe): CheckableState = look_controls::empty_toggle_part,
        /// A persistent part inside the checkbox indicator for the indeterminate mark.
        checkbox_mixed(checkbox_mixed_recipe): CheckableState = look_controls::empty_toggle_part,
        switch(switch_recipe): CheckableState = look::switch,
        /// The switch track. Default recipes draw the track and thumb in the root.
        switch_track(switch_track_recipe): CheckableState = look_controls::empty_toggle_part,
        /// A persistent part inside the track for the moving thumb.
        switch_thumb(switch_thumb_recipe): CheckableState = look_controls::empty_toggle_part,
        /// A [`toggle_button`](super::toggle) button: the icon and the label in a row.
        toggle(toggle_recipe): ToggleButtonState = look::toggle_button,
        text_input(text_input_recipe): TextInputState = look::text_input,
        text_area(text_area_recipe): TextInputState = look::text_area,
    }
    tables {
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
        root(root_recipe): CarouselState = look_files::carousel_root,
        viewport(viewport_recipe): CarouselState = look_files::carousel_viewport,
        track(track_recipe): CarouselState = look_files::carousel_track,
        slide(slide_recipe): CarouselState = look_files::carousel_slide,
        previous(previous_recipe): CarouselState = look_files::carousel_control,
        next(next_recipe): CarouselState = look_files::carousel_control,
    }
}
recipe_table! {
    /// Styles for a file drop target and its feedback.
    DropzoneRecipes {
        root(root_recipe): DropzoneState = look_files::dropzone_root,
        icon(icon_recipe): DropzoneState = look_files::dropzone_icon,
        label(label_recipe): DropzoneState = look_files::dropzone_label,
        hint(hint_recipe): DropzoneState = look_files::dropzone_hint,
        status(status_recipe): DropzoneState = look_files::dropzone_status,
    }
}
recipe_table! {
    /// Toolbar surface recipes.
    ToolbarRecipes {
        root(root_recipe): crate::ToolbarState = crate::toolbar::root_style,
    }
}
recipe_table! {
    /// Toggle group surface and item recipes.
    ToggleGroupRecipes {
        root(root_recipe): crate::ToggleGroupState = crate::toggle_group::root_style,
        item(item_recipe): crate::ToggleGroupItemState = crate::toggle_group::item_style,
    }
}
recipe_table! {
    /// Joined action group surface and auxiliary cell recipes.
    ButtonGroupRecipes {
        root(root_recipe): crate::ButtonGroupState = crate::button_group::root_style,
        member(member_recipe): crate::ButtonGroupMemberState = crate::button_group::member_style,
        separator(separator_recipe): crate::ButtonGroupState = crate::button_group::separator_style,
        text(text_recipe): crate::ButtonGroupState = crate::button_group::text_style,
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
        title(title_recipe): AlertState = look_feedback::alert_title,
        description(description_recipe): AlertState = look_feedback::alert_description,
        icon(icon_recipe): AlertState = look_feedback::alert_icon,
        actions(actions_recipe): AlertState = look_feedback::alert_actions,
        content(content_recipe): AlertState = look_feedback::alert_content,
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
    }
}
recipe_table! {
    /// Recipes for a labelled group of controls and its text adornments.
    InputGroupRecipes {
        root(root_recipe): InputGroupState = look_feedback::group_root,
        label(label_recipe): InputGroupState = look_feedback::group_label,
        members(members_recipe): InputGroupState = look_feedback::group_members,
        member(member_recipe): InputGroupMemberState = look_feedback::group_member,
        addon(addon_recipe): InputGroupState = look_feedback::group_addon,
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
        root(root_recipe): PartState = look_static_collections::list,
        item(item_recipe): PartState = look_static_collections::item,
        body(body_recipe): PartState = look_static_collections::item_body,
        content(content_recipe): PartState = look_static_collections::content,
        title(title_recipe): PartState = look_static_collections::title,
        description(description_recipe): PartState = look_static_collections::description,
        media(media_recipe): PartState = look_static_collections::media,
        actions(actions_recipe): PartState = look_static_collections::actions,
        header(header_recipe): PartState = look_static_collections::band,
        footer(footer_recipe): PartState = look_static_collections::band,
    }
}
recipe_table! {
    /// Page control and gap recipes.
    PaginationRecipes {
        root(root_recipe): () = crate::look_navigation::root,
        item(item_recipe): bool = crate::look_navigation::current,
        ellipsis(ellipsis_recipe): () = crate::look_navigation::part,
    }
}
recipe_table! {
    /// Breadcrumb landmark, item and decorative separator recipes.
    BreadcrumbRecipes {
        root(root_recipe): () = crate::look_navigation::breadcrumb_root,
        item(item_recipe): bool = crate::look_navigation::current,
        separator(separator_recipe): () = crate::look_navigation::part,
    }
}
recipe_table! {
    /// Progress list and individual step recipes.
    StepperRecipes {
        root(root_recipe): () = crate::look_navigation::stepper_root,
        item(item_recipe): crate::StepperItemState = crate::look_navigation::stepper_item,
        indicator(indicator_recipe): crate::StepperItemState = crate::look_navigation::part,
        label(label_recipe): crate::StepperItemState = crate::look_navigation::part,
        description(description_recipe): crate::StepperItemState = crate::look_navigation::part,
        content(content_recipe): crate::StepperItemState = crate::look_navigation::part,
        end_content(end_content_recipe): crate::StepperItemState = crate::look_navigation::part,
    }
}
recipe_table! {
    /// Navigation landmark, panel trigger and panel surface recipes.
    NavigationMenuRecipes {
        root(root_recipe): kivact_interaction::NavOpen = crate::look_navigation::navigation_root,
        trigger(trigger_recipe): kivact_interaction::NavOpen = crate::look_navigation::part,
        panel(panel_recipe): kivact_interaction::NavOpen = crate::look_navigation::panel,
    }
}
