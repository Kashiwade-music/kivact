//! Role: Interactive styled component catalog for kivact-ui-game.
#![warn(redundant_imports)]
mod game;
mod messages;
use kivact::prelude::*;

pub use messages::{locale, messages};

/// Mount the complete catalog through public component APIs.
pub fn catalog() -> impl IntoView {
    kivact::ui::toaster().max_visible(4);
    game::catalog()
}

/// Application-owned, offline OFL font bytes for the catalog and regressions.
pub fn fonts() -> &'static [&'static [u8]] {
    &[
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-SemiBold.ttf"),
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-Bold.ttf"),
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-BoldItalic.ttf"),
        include_bytes!("../assets/fonts/barlowcondensed/BarlowCondensed-ExtraBoldItalic.ttf"),
        include_bytes!("../assets/fonts/jetbrainsmono/JetBrainsMono[wght].ttf"),
        include_bytes!("../assets/fonts/notosansjp/NotoSansJP[wght].ttf"),
    ]
}
