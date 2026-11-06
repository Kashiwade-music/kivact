//! Role: Pure source-highlighting data and Basic code presentation variants.
mod highlight;
mod lines;
mod rust;
mod view;
pub use highlight::rust_lines;
pub use lines::{CodeLine, CodeSpan, CodeTone};
pub use rust::rust_source;
pub(crate) use view::recipes;
pub use view::{CodeBlockLook, CodeBlockSize};
