//! Role: The catalogs' shared offline OFL fonts.

/// Inter, Geist Mono and Noto Sans JP (subset), for the catalogs and their tests.
pub static FONTS: &[&[u8]] = &[
    include_bytes!("../../shared-assets/fonts/inter/Inter[opsz,wght].ttf"),
    include_bytes!("../../shared-assets/fonts/geistmono/GeistMono[wght].ttf"),
    include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"),
];
