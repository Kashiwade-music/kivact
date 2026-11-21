//! Role: The Soft component catalog: one page per component, each with its demo and the code that makes it.
mod pages;

use catalog_shell::CatalogTheme;
use kivact::style::{Theme, ThemeMode};

pub use pages::pages;

static FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/figtree/Figtree-Variable.ttf"),
    include_bytes!("../assets/fonts/figtree/Figtree-Italic-Variable.ttf"),
];

/// The Soft theme in `mode`, with the catalog's Figtree fonts.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_soft::theme(mode).with_fonts(FONTS.iter().copied())
}

/// The catalog's theme, for `catalog_shell::catalog`.
pub const THEME: CatalogTheme = CatalogTheme::new(theme);

/// The page shown first.
pub const START: &str = "button";
