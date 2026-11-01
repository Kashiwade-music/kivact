//! Role: Entry point of the Basic kit catalog app.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_ui_basic_catalog::{catalog, fonts};

fn main() -> kivact::Result<()> {
    let mut app = App::new("Kivact UI · Basic").window_size(1440, 900);
    for &font in fonts() {
        app = app.font(font);
    }
    app.run(catalog)
}
