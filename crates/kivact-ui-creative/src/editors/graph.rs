//! Role: Public graph builder and reactive assembly of graph domain operations.

pub(crate) mod a11y;
pub(crate) mod edit;
pub(crate) mod gesture;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod paint;
mod view;
pub(crate) mod view_state;

pub use view::{GraphEditor, graph_editor};
