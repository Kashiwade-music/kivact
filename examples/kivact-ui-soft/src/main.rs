//! Role: Entry point of the Soft kit catalog app.
use kivact::App;
use kivact_ui_soft_catalog::{START, THEME, pages};

fn main() -> kivact::Result<()> {
    App::new("Kivact UI · Soft")
        .window_size(1440, 900)
        .run(move || catalog_shell::catalog("Soft", pages(), THEME).start(START))
}
