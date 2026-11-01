//! Role: Entry point of the Pop kit catalog app.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact_ui_pop_catalog::{catalog, fonts, locale, messages};

fn main() -> kivact::Result<()> {
    let mut app = App::new("Kivact · Official UI · Pop")
        .locale(locale())
        .messages(messages());
    for &font in fonts() {
        app = app.font(font);
    }
    app.run(catalog)
}
