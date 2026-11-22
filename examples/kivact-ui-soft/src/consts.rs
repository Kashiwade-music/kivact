//! Role: The catalog's theme and the page it opens on.
use crate::theme::theme;
use catalog_shell::CatalogTheme;

/// The catalog's theme, for `catalog_shell::catalog`.
pub const THEME: CatalogTheme = CatalogTheme::new(theme);

/// The page shown first.
pub const START: &str = "button";
