//! Role: The catalog's offline OFL fonts: Inter, Barlow and Noto Sans JP (subset).

/// The fonts the Pop catalog adds to its theme, shared by the app and its tests.
pub static FONTS: &[&[u8]] = &[
    include_bytes!("../../shared-assets/fonts/inter/Inter[opsz,wght].ttf"),
    include_bytes!("../assets/fonts/barlow/Barlow-BoldItalic.ttf"),
    include_bytes!("../assets/fonts/barlow/Barlow-ExtraBoldItalic.ttf"),
    include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"),
];
