//! Role: The Basic kit catalog: one documentation page per component.
//!
//! [`pages`] lists the pages and [`THEME`] is the kit's theme with the catalog's fonts; the
//! shared shell of `catalog-shell` draws them with a searchable list and a theme toggle.

mod consts;
mod docs;
mod theme;

pub use consts::START;
pub use docs::pages;
pub use theme::{THEME, theme};
