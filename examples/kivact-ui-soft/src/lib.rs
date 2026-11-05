//! Role: Expose the Soft component catalog, its pages and embedded fonts.
#![deny(clippy::too_many_lines, clippy::disallowed_types)]
mod docs;
mod fonts;
mod look;
mod model;
mod page;
mod pages;
mod shell;

pub use fonts::fonts;
pub use pages::{PageInfo, pages};
pub use shell::{catalog, catalog_at, catalog_with_mode};
