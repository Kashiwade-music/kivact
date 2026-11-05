//! Role: The catalog's offline OFL font bytes, for the app and its tests.

/// Application-owned, offline OFL font bytes for the catalog and regressions.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../../shared-assets/fonts/geist/Geist[wght].ttf"),
        include_bytes!("../../shared-assets/fonts/geistmono/GeistMono[wght].ttf"),
        include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}
