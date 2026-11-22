//! Role: The Basic kit catalog: one documentation page per component, shown by the catalog shell.
//!
//! [`pages`] lists the pages and [`THEME`] is the kit's theme with the catalog's fonts; the
//! shared shell of `catalog-shell` draws them with a searchable list and a theme toggle.

mod docs;

use catalog_shell::CatalogTheme;
use kivact::style::{Theme, ThemeMode};

pub use docs::pages;

/// The Basic kit's theme in `mode`, with the catalog fonts and the text family of the catalog.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_basic::theme(mode)
        .with_fonts(catalog_shell::FONTS.iter().copied())
        .set(
            &kivact::style::font::FAMILY_VAR,
            kivact::style::FontFamily::from("Inter, 'Noto Sans JP', sans-serif"),
        )
}

/// The catalog's theme.
pub const THEME: CatalogTheme = CatalogTheme::new(theme);

/// The slug of the page shown first.
pub const START: &str = "button";
