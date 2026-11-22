//! Role: The sidebar: its provider, panel, rail, trigger, groups and menus.
//! With their recipes and theme variables.
//!
//! The provider draws the sidebar beside the page and shares its open
//! state, side, frame and collapsing with the parts below it. A part used
//! outside a provider behaves as an expanded sidebar.

pub(crate) mod look;
pub mod vars;

mod menu;
mod panel;
mod recipes;
mod resize;
mod state;
mod trigger;

pub use menu::{
    SidebarGroup, SidebarInput, SidebarMenu, SidebarMenuButton, SidebarMenuSkeleton,
    SidebarMenuSubButton, SidebarSeparator, sidebar_group, sidebar_input, sidebar_menu,
    sidebar_menu_button, sidebar_menu_skeleton, sidebar_menu_sub_button, sidebar_separator,
};
pub use panel::{Sidebar, SidebarProvider, sidebar, sidebar_provider};
pub use state::{
    SidebarCollapsible, SidebarFrame, SidebarHandle, SidebarMenuButtonSize, SidebarResize,
    SidebarSide, use_sidebar,
};
pub use trigger::{SidebarTrigger, sidebar_trigger};
