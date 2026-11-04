//! Role: Supply offline Figtree font bytes for the catalog and its tests.

/// Upright and italic Figtree variable fonts.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../assets/fonts/figtree/Figtree-Variable.ttf"),
        include_bytes!("../assets/fonts/figtree/Figtree-Italic-Variable.ttf"),
    ]
}
