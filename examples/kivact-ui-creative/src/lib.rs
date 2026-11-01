//! Role: Interactive styled component catalog for kivact-ui-creative.
#![warn(redundant_imports)]
mod creative;
use kivact::prelude::*;

/// Mount the complete catalog through public component APIs.
pub fn catalog() -> impl IntoView {
    creative::catalog()
}

/// Application-owned, offline OFL font bytes for the catalog and regressions.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../assets/fonts/geist/Geist[wght].ttf"),
        include_bytes!("../assets/fonts/geistmono/GeistMono[wght].ttf"),
        include_bytes!("../assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}
