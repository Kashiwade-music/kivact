//! Role: Entry point of the Game kit catalog app.

use kivact::prelude::*;
use kivact_ui_game_catalog::{START, THEME, pages};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Game")
        .window_size(1440, 900)
        .run(move || catalog_shell::catalog("Game", pages(), THEME).start(START))
}
