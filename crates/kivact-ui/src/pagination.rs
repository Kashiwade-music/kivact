//! Role: Render stable keyed page controls and an atomic editable page draft.

mod draft;
pub(crate) mod look;
mod view;

pub use look::{PageItem, PaginationMode};
pub use view::{Pagination, PaginationWindow, pagination};
