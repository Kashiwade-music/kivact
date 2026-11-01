//! Role: Document-style component catalog for kivact-ui-basic.
//!
//! A top bar with the theme toggle, a searchable component list and one
//! documentation page per component. Every code block shows the source of
//! the demo above it.
#![warn(redundant_imports)]
mod docs;
mod fonts;
mod look;
mod model;
mod page;
mod pages;
mod shell;

pub use fonts::fonts;
pub use pages::{ExampleInfo, PageInfo, pages};
pub use shell::{catalog, catalog_with_mode};
