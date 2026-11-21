//! Role: The Creative kit catalog: its pages, its theme and the page shown first.

mod pages;

use catalog_shell::CatalogTheme;
use kivact::style::{Theme, ThemeMode};

pub use pages::pages;

/// The catalog's fonts: Geist, Geist Mono and Noto Sans JP (subset).
static FONTS: &[&[u8]] = &[
    include_bytes!("../../shared-assets/fonts/geist/Geist[wght].ttf"),
    include_bytes!("../../shared-assets/fonts/geistmono/GeistMono[wght].ttf"),
    include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"),
];

/// The catalog theme in `mode`: the kit's theme with the catalog fonts.
pub fn theme(mode: ThemeMode) -> Theme {
    kivact_ui_creative::theme(mode).with_fonts(FONTS.iter().copied())
}

/// The catalog's theme, for [`catalog_shell::catalog`].
pub const THEME: CatalogTheme = CatalogTheme::new(theme);

/// The slug of the page shown first.
pub const START: &str = "buttons";
