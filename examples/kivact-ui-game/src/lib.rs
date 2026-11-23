//! Role: The game kit's catalog: eleven pages of its components.
//!
//! Each page is a file of `src/pages/`: its demo sits between `// example:` and `// end example`,
//! so the code on screen is the code that runs.

mod consts;
mod fonts;
mod list;
mod pages;
mod support;
mod theme;

pub use consts::START;
pub use list::pages;
pub use theme::{THEME, theme};
