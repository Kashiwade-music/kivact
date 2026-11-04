//! Role: Expose sidebar navigation, nested items and collapsed rail.

pub(crate) mod context;
pub(crate) mod items;
mod shell;

pub use items::{ItemCollapse, SideNavItem, SideNavSection, side_nav_item, side_nav_section};
pub use shell::{
    SideNav, SideNavCollapse, SideNavCollapseButton, SideNavResize, side_nav,
    side_nav_collapse_button,
};
