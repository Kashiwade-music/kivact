//! Role: The DevTools app's offline font bytes, for the app and its tests.

/// The application-owned, offline font files of the app.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../../../examples/shared-assets/fonts/inter/Inter[opsz,wght].ttf"),
        include_bytes!("../../../examples/shared-assets/fonts/geistmono/GeistMono[wght].ttf"),
    ]
}
