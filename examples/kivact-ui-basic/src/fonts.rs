//! Role: The catalog's offline OFL font bytes, for the app and its tests.

/// Application-owned, offline OFL font bytes for the catalog and its tests.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../assets/fonts/inter/Inter[opsz,wght].ttf"),
        include_bytes!("../../shared-assets/fonts/geistmono/GeistMono[wght].ttf"),
        include_bytes!("../assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}
