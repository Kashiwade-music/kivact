//! Role: The Pop kit catalog: one page per component family, shown by the shared catalog shell.
//!
//! Each page file defines its demo between `// example: <id>` and `// end example`, so the code
//! on screen is the code that runs.

mod fonts;
mod pages;

use catalog_shell::{CatalogTheme, Page};
use kivact::style::{Theme, ThemeMode};

/// The slug of the page the catalog opens on.
pub const START: &str = "overview";

/// The catalog theme: [`theme`] with the catalog's fonts.
pub const THEME: CatalogTheme = CatalogTheme::new(theme);

/// The Pop theme in `mode`, with the catalog's fonts added.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_pop::theme(mode).with_fonts(fonts::FONTS.iter().copied())
}

/// The catalog's pages, in list order.
pub fn pages() -> Vec<Page> {
    vec![
        pages::overview::page(),
        pages::tokens::page(),
        pages::buttons::page(),
        pages::forms::page(),
        pages::selection::page(),
        pages::display::page(),
        pages::table::page(),
        pages::feedback::page(),
        pages::overlays::page(),
        pages::navigation::page(),
        pages::messaging::page(),
        pages::schedule::page(),
        pages::cards::page(),
    ]
}
