//! Role: The catalog's offline OFL font bytes, for the app and its tests.

/// Application-owned, offline OFL font bytes for the catalog and regressions.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-SemiBold.ttf"),
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-Bold.ttf"),
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-BoldItalic.ttf"),
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-ExtraBoldItalic.ttf"),
        include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono[wght].ttf"),
        include_bytes!("../assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}
