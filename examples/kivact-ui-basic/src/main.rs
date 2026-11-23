//! Role: Entry point of the Basic kit catalog app.

use kivact::prelude::*;
use kivact_ui_basic_catalog::{START, THEME, pages};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Basic")
        .window_size(1440, 900)
        .run(move || catalog_shell::catalog("Basic", pages(), THEME).start(START))
}
