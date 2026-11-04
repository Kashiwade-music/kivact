//! Role: Expose sidebar navigation, nested items and collapsed rail.

mod context;
mod items;
mod shell;

pub(crate) use context::nav_is_rail;
pub use items::{ItemCollapse, SideNavItem, SideNavSection, side_nav_item, side_nav_section};
pub(crate) use items::{logical_row, side_row_style};
pub use shell::{
    SideNav, SideNavCollapse, SideNavCollapseButton, SideNavResize, side_nav,
    side_nav_collapse_button,
};
