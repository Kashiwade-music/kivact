//! Role: A typed node editor with application-owned models and shared pan/zoom.

pub(crate) mod a11y;
pub(crate) mod edit;
pub(crate) mod gesture;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod paint;
pub(crate) mod parameters;
mod view;
pub(crate) mod view_state;

pub use view::{NodeEditor, node_editor};
