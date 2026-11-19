//! Role: Present selectable code with syntax, numbering and clipboard feedback.

pub(crate) mod editing;
pub(crate) mod gutter;
pub(crate) mod look;
pub(crate) mod tokens;
mod view;

pub use look::CodeCollapse;
pub use view::{CodeBlock, code_block};
