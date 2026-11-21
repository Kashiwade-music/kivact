//! Role: The shared shell of the Kivact UI kit catalogs: page list, search, theme toggle and pages.
//!
//! A catalog is a list of [`Page`]s and a theme function; [`catalog`] shows them
//! with a searchable list, a theme toggle and the page area. A page file defines
//! its demos between `// example: <id>` and `// end example`, so the code on
//! screen is the code that runs.

#[cfg(feature = "test")]
pub mod check;
mod fonts;
mod model;
mod page;
mod shell;

pub use fonts::FONTS;
pub use model::{Example, Page, PreviewAlign};
pub use shell::{CatalogTheme, catalog, open_page};
