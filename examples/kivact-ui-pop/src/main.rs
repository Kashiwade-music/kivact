//! Entry point of the Pop kit catalog app.

use kivact::prelude::*;
use kivact_ui_pop_catalog::{catalog, fonts, locale, messages};

fn main() -> kivact::Result<()> {
    App::new("Kivact · Official UI · Pop")
        .locale(locale())
        .messages(messages())
        .fonts(fonts().iter().copied())
        .run(catalog)
}
