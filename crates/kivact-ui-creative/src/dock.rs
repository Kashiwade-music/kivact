//! Role: A docking workspace: splits of tabbed containers with retained panels.
//!
//! The layout is an application-owned `Binding<DockLayout<K>>`. Containers,
//! splitters and panel contents are siblings positioned from the model, so a
//! panel's view stays mounted while its tab switches, moves to another
//! container or the tree is restructured.

mod chrome;
mod data;
mod geometry;
mod items;
mod model;
mod operations;
mod view;

pub use model::{
    DockAxis, DockChild, DockLayout, DockNode, DockPlacement, DockSide, DockTab, DockTabs,
};
pub use view::{Dock, dock};
