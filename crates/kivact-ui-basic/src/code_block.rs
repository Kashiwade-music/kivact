//! Role: A highlighted, selectable code listing with title, copy action and footer.

mod lines;
mod rust;
mod view;

pub use lines::{CodeLine, CodeSpan, CodeTone, code_lines};
pub use rust::rust_lines;
pub use view::{CodeBlock, CodeBlockSize, code_block};
