//! Role: The catalog's theme and the page it opens on.
use crate::theme::theme;
use catalog_shell::CatalogTheme;

/// The slug of the page the catalog opens on.
pub const START: &str = "overview";

/// The catalog theme: [`theme`] with the catalog's fonts.
pub const THEME: CatalogTheme = CatalogTheme::new(theme);
