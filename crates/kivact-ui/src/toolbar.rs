//! Role: Compose named toolbar regions with native roving focus.
pub(crate) mod look;
pub use look::ToolbarEdge;
mod view;
pub use view::{Toolbar, toolbar};
