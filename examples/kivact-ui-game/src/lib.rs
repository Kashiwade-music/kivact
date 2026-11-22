//! Role: The game kit's catalog: eleven pages of its components, shown by the shared catalog shell.
//!
//! Each page is a file of `src/pages/`: its demo sits between `// example:` and `// end example`,
//! so the code on screen is the code that runs.

mod fonts;
mod pages;
mod support;

use catalog_shell::{CatalogTheme, Page};
use kivact::style::{FontFamily, Theme, ThemeMode};
use kivact_ui_game::{Density, ThemeOptions};

/// The first page shown.
pub const START: &str = "foundations";

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

/// The eleven pages in the order of the list.
pub fn pages() -> Vec<Page> {
    vec![
        pages::foundations::page(),
        pages::buttons::page(),
        pages::inputs::page(),
        pages::selection::page(),
        pages::navigation::page(),
        pages::data_display::page(),
        pages::data_table::page(),
        pages::feedback::page(),
        pages::overlays::page(),
        pages::native_layout::page(),
        pages::effects::page(),
    ]
}

/// The theme options of the catalog: the default density and the two families.
pub(crate) fn options() -> ThemeOptions {
    ThemeOptions {
        density: Density::Comfortable,
        display_font: Some("Barlow Condensed".into()),
        mono_font: Some("JetBrains Mono".into()),
    }
}

use fonts::FONTS;
