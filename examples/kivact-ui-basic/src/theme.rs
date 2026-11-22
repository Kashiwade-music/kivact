//! Role: The Basic catalog theme: the kit's theme with the catalog fonts.

use catalog_shell::CatalogTheme;
use kivact::style::{Theme, ThemeMode};

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
