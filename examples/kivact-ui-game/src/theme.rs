//! Role: The game catalog theme: the game kit's theme with the catalog fonts.

use crate::fonts::FONTS;
use catalog_shell::CatalogTheme;
use kivact::style::{FontFamily, Theme, ThemeMode};
use kivact_ui_game::{Density, ThemeOptions};

/// The game theme with the catalog's fonts: Barlow Condensed for headings and
/// buttons, JetBrains Mono for codes.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_game::theme_with(mode, options())
        .set(
            &kivact::style::font::FAMILY_VAR,
            FontFamily::from("Noto Sans JP"),
        )
        .with_fonts(FONTS.iter().copied())
}

/// The catalog's theme, shared by the shell and the tests.
pub const THEME: CatalogTheme = CatalogTheme::new(theme);

/// The theme options of the catalog: the default density and the two families.
pub(crate) fn options() -> ThemeOptions {
    ThemeOptions {
        density: Density::Comfortable,
        display_font: Some("Barlow Condensed".into()),
        mono_font: Some("JetBrains Mono".into()),
    }
}
