//! Role: Entry point of the Creative kit catalog app.

use kivact::prelude::*;
use kivact_ui_creative_catalog::{START, THEME, pages};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Creative")
        .window_size(1440, 900)
        .run(move || catalog_shell::catalog("Creative", pages(), THEME).start(START))
}
