//! Role: Compose semantic static lists with fixed child positions and header content.

pub(crate) mod context;
pub(crate) mod look;
pub(crate) mod row;
mod view;

pub use look::{ItemListMarker, ItemPosition};
pub use view::{ItemList, item_list};
