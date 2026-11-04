//! Role: Unstyled behavior and semantics of a splitter between two panes.
//!
//! [`use_splitter`] edits the leading pane's size with the same normalized
//! writer as a [`ScalarApi`], and its drag is a window-coordinate
//! [`DragApi`](crate::DragApi) made one edit with `DragApi::edit`. Every kit's grip,
//! pane, dock split and column resize shares this part instead of copying
//! its keys: the table's [`resize`](crate::DataTableApi::resize) is a splitter, and
//! [`SplitterApi::handle`] is what `kivact::ui::splitter` draws. The cursor is
//! the caller's style.

mod handle;
mod machine;
mod options;
pub use handle::{SplitterApi, SplitterState, use_splitter};
