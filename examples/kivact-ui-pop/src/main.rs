//! Role: Entry point of the Pop kit catalog app.

use kivact::prelude::*;
use kivact_ui_pop_catalog::{START, THEME, pages};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Pop")
        .window_size(1440, 900)
        .run(move || catalog_shell::catalog("Pop", pages(), THEME).start(START))
}
