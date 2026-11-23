//! Role: The catalog's fonts: Geist, Geist Mono and Noto Sans JP (subset).

/// The catalog's fonts: Geist, Geist Mono and Noto Sans JP (subset).
pub(crate) static FONTS: &[&[u8]] = &[
    include_bytes!("../../shared-assets/fonts/geist/Geist[wght].ttf"),
    include_bytes!("../../shared-assets/fonts/geistmono/GeistMono[wght].ttf"),
    include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"),
];
