//! Role: The Lyra sidebar: provider, panel, rail, trigger and menu parts.

mod menu;
mod panel;
pub(crate) mod state;
mod trigger;

pub use menu::{
    SidebarGroup, SidebarMenuButton, SidebarMenuButtonSize, SidebarMenuSubButton, sidebar_group,
    sidebar_input, sidebar_menu, sidebar_menu_button, sidebar_menu_skeleton,
    sidebar_menu_sub_button, sidebar_separator,
};
pub use panel::{Sidebar, sidebar, sidebar_provider};
pub use state::{SidebarCollapsible, SidebarSide, SidebarVariant};
pub use trigger::{SidebarTrigger, sidebar_trigger};
