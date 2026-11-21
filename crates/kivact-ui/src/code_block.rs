//! Role: Present selectable code with syntax, numbering and clipboard feedback.

pub(crate) mod editing;
pub(crate) mod gutter;
pub(crate) mod look;
mod rust;
mod view;

pub use look::CodeCollapse;
pub use rust::rust_tokens;
pub use view::{CodeBlock, Highlighter, code_block, highlighter};
