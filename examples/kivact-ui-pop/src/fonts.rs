//! Role: The catalog's offline OFL font bytes, for the app and its tests.

/// Application-owned, offline OFL font bytes for the catalog and regressions.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../../shared-assets/fonts/inter/Inter[opsz,wght].ttf"),
        include_bytes!("../assets/fonts/barlow/Barlow-BoldItalic.ttf"),
        include_bytes!("../assets/fonts/barlow/Barlow-ExtraBoldItalic.ttf"),
        include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}
