//! Creative example application entry point.

use kivact::prelude::*;
use kivact_ui_creative_catalog::{catalog, fonts};

fn main() -> kivact::Result<()> {
    let mut app = App::new("Kivact · Official UI · Creative");
    for &font in fonts() {
        app = app.font(font);
    }
    app.run(catalog)
}
