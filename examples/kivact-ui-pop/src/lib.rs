//! Role: The Pop kit catalog: one page per component family, shown by the shared catalog
//! shell.
//!
//! Each page file defines its demo between `// example: <id>` and `// end example`, so the code
//! on screen is the code that runs.

mod catalog;
mod consts;
mod fonts;
mod pages;
mod theme;

pub use catalog::pages;
pub use consts::{START, THEME};
pub use theme::theme;
