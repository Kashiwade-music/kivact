//! Role: The catalog theme: the kit's theme with the catalog fonts.

use crate::fonts::FONTS;
use catalog_shell::CatalogTheme;
use kivact::style::{Theme, ThemeMode};

/// The catalog theme in `mode`: the kit's theme with the catalog fonts.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_creative::theme(mode).with_fonts(FONTS.iter().copied())
}

/// The catalog's theme, for [`catalog_shell::catalog`].
pub const THEME: CatalogTheme = CatalogTheme::new(theme);
