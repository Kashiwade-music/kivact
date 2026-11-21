//! Role: The sidebar's recipe states and recipe table.

use super::recipes;
use super::state::{SidebarCollapsible, SidebarFrame, SidebarMenuButtonSize, SidebarSide};
use crate::recipe::recipe_table;

/// What the panel-level parts of a sidebar read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarState {
    /// Whether the sidebar is open.
    pub open: bool,
    /// Whether the sidebar is an icon column (`Icon` and closed).
    pub icon_only: bool,
    /// The side of the window.
    pub side: SidebarSide,
    /// The frame.
    pub frame: SidebarFrame,
    /// What closing does.
    pub collapsible: SidebarCollapsible,
}

/// What a row of a sidebar menu (and its badge, action and sub list) reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarItemState {
    /// Whether the row is the current page.
    pub active: bool,
    /// Whether the sidebar is an icon column.
    pub icon_only: bool,
    /// Whether the row has an action control.
    pub has_action: bool,
    /// Whether the reading direction is right to left.
    pub rtl: bool,
    /// Whether the row is disabled: it takes no press.
    pub disabled: bool,
    /// The row's height.
    pub size: SidebarMenuButtonSize,
    /// Whether the row's sub list is open, or `None` when the row has no toggle.
    pub expanded: Option<bool>,
}

recipe_table! {
    /// The recipes of the sidebar components.
    SidebarTheme {
        /// The provider's row, which holds the sidebar and the page beside it.
        root(root_recipe): SidebarState = recipes::provider,
        /// The box of a sidebar that takes its width in the layout.
        sidebar(sidebar_recipe): SidebarState = recipes::sidebar,
        /// The panel of a sidebar, framed by its frame.
        panel(panel_recipe): SidebarState = recipes::panel,
        /// The top section of the panel.
        header(header_recipe): SidebarState = recipes::section,
        /// The scrolling middle section of the panel.
        content(content_recipe): SidebarState = recipes::content,
        /// The bottom section of the panel.
        footer(footer_recipe): SidebarState = recipes::section,
        /// The strip on the inner edge that toggles the sidebar.
        rail(rail_recipe): SidebarState = recipes::rail,
        /// The page beside an inset sidebar.
        inset(inset_recipe): SidebarState = recipes::inset,
        /// The button that toggles the sidebar.
        trigger(trigger_recipe): SidebarState = recipes::trigger,
        /// The strip on the inner edge that drags a resizable sidebar's width.
        resize_handle(resize_handle_recipe): SidebarState = recipes::resize_handle,
        /// A labelled group of the sidebar.
        group(group_recipe): SidebarState = recipes::group,
        /// The label row of a group.
        group_label(group_label_recipe): SidebarState = recipes::group_label,
        /// The line under a group's label.
        group_description(group_description_recipe): SidebarState = recipes::group_description,
        /// The action at the end of a group's label row.
        group_action(group_action_recipe): SidebarState = recipes::group_action,
        /// The body of a group.
        group_content(group_content_recipe): SidebarState = recipes::group_content,
        /// A menu: a list of rows.
        menu(menu_recipe): SidebarState = recipes::menu,
        /// A horizontal rule.
        separator(separator_recipe): SidebarState = recipes::separator,
        /// A text field in the sidebar.
        input(input_recipe): SidebarState = recipes::input,
        /// A loading placeholder row.
        skeleton(skeleton_recipe): SidebarState = recipes::skeleton,
        /// The wrapper of a menu row.
        menu_item(menu_item_recipe): SidebarItemState = recipes::menu_item,
        /// The button of a menu row.
        menu_button(menu_button_recipe): SidebarItemState = recipes::menu_button,
        /// The icon of a menu row.
        menu_button_icon(menu_button_icon_recipe): SidebarItemState = crate::look_common::part,
        /// The label of a menu row.
        menu_button_label(menu_button_label_recipe): SidebarItemState = recipes::menu_button_label,
        /// The element at the end of a menu row, before its badge.
        menu_button_end(menu_button_end_recipe): SidebarItemState = recipes::menu_button_end,
        /// The chevron of a menu row with a sub list: down while the list is open.
        menu_button_chevron(menu_button_chevron_recipe): SidebarItemState = recipes::menu_button_chevron,
        /// The count at the end of a menu row.
        menu_badge(menu_badge_recipe): SidebarItemState = recipes::menu_badge,
        /// The label shown beside an icon-only menu row.
        menu_tooltip(menu_tooltip_recipe): SidebarItemState = recipes::menu_tooltip,
        /// The action at the end of a menu row.
        menu_action(menu_action_recipe): SidebarItemState = recipes::menu_action,
        /// The sub list under a menu row.
        menu_sub(menu_sub_recipe): SidebarItemState = recipes::menu_sub,
        /// A link in a sub list.
        sub_button(sub_button_recipe): SidebarItemState = recipes::sub_button,
    }
}
