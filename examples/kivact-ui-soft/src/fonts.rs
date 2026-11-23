//! Role: The Figtree fonts that the catalog adds to the Soft theme.

/// The fonts the Soft catalog adds to its theme, shared by the app and its tests.
pub(crate) static FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/figtree/Figtree-Variable.ttf"),
    include_bytes!("../assets/fonts/figtree/Figtree-Italic-Variable.ttf"),
];
