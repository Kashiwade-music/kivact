//! Role: Interactive styled component catalog for kivact-ui-pop.
#![warn(redundant_imports)]
mod messages;
mod pop;
use kivact::prelude::*;

pub use messages::{locale, messages};

/// Mount the complete catalog through public component APIs.
pub fn catalog() -> impl IntoView {
    pop::catalog()
}

/// Application-owned, offline OFL font bytes for the catalog and regressions.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../assets/fonts/inter/Inter[opsz,wght].ttf"),
        include_bytes!("../assets/fonts/barlow/Barlow-BoldItalic.ttf"),
        include_bytes!("../assets/fonts/barlow/Barlow-ExtraBoldItalic.ttf"),
        include_bytes!("../assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}
