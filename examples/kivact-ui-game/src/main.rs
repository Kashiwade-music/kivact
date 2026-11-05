//! Role: Game example application entry point.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_ui_game_catalog::{catalog, fonts, locale, messages};

fn main() -> kivact::Result<()> {
    let app = App::new("Kivact · Official UI · Game")
        .locale(locale())
        .messages(messages());
    app.fonts(fonts().iter().copied()).run(catalog)
}
