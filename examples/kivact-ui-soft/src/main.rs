//! Role: Open the Soft component catalog with application-owned fonts.
#![deny(clippy::too_many_lines, clippy::disallowed_types)]
use kivact::App;
use kivact_ui_soft_catalog::{catalog, fonts};

fn main() -> kivact::Result<()> {
    let mut app = App::new("Kivact UI · Soft").window_size(1440, 900);
    for &font in fonts() {
        app = app.font(font);
    }
    app.run(catalog)
}
