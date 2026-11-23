//! Role: The game catalog's offline OFL fonts: Barlow, JetBrains Mono, Noto Sans JP.
//! The Noto Sans JP file is a subset.

/// The fonts the catalog registers with its theme.
pub(crate) static FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-SemiBold.ttf"),
    include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-Bold.ttf"),
    include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-BoldItalic.ttf"),
    include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-ExtraBoldItalic.ttf"),
    include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono[wght].ttf"),
    include_bytes!("../../shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"),
];
